/// Currency conversion backed by the Frankfurter API (ECB data, free, no auth).
///
/// Architecture:
/// - Live fetch from https://api.frankfurter.dev/v1/latest?from=USD (5s timeout)
/// - 24h file cache at ~/.cache/heimdall/fx.json
/// - Graceful fallback: live → cached → USD (unchanged)
/// - Storage is always USD nanos; this module is display-only.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const FRANKFURTER_URL: &str = "https://api.frankfurter.dev/v1/latest?from=USD";
const FETCH_TIMEOUT_SECS: u64 = 5;

// ── Public types ────────────────────────────────────────────────────────────

/// The result of a USD→target conversion.
#[derive(Debug, Clone)]
pub struct ConvertResult {
    /// Converted amount in target currency.
    pub amount: f64,
    /// ISO 4217 code, or "USD" on fallback.
    pub currency: String,
    /// How the rate was sourced.
    pub source: RateSource,
}

/// How the exchange rate was obtained.
#[derive(Debug, Clone)]
pub enum RateSource {
    /// Rate fetched live from Frankfurter just now.
    Live,
    /// Rate loaded from the on-disk cache.
    Cached {
        /// How old the cache entry is (hours).
        age_hours: f64,
    },
    /// No rate available; amount returned unchanged in USD.
    Fallback,
}

impl RateSource {
    /// Lowercase string suitable for JSON serialisation.
    pub fn as_str(&self) -> &'static str {
        match self {
            RateSource::Live => "live",
            RateSource::Cached { .. } => "cached",
            RateSource::Fallback => "fallback",
        }
    }

    /// Age in hours for Cached variant; None otherwise.
    pub fn age_hours(&self) -> Option<f64> {
        match self {
            RateSource::Cached { age_hours } => Some(*age_hours),
            _ => None,
        }
    }
}

// ── Internal cache types ────────────────────────────────────────────────────

/// What we persist to ~/.cache/heimdall/fx.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatesSnapshot {
    /// RFC 3339 timestamp of when rates were fetched.
    pub fetched_at: String,
    /// Base currency (always "USD" for our use-case).
    pub base: String,
    /// Map of ISO 4217 code → rate relative to base.
    pub rates: HashMap<String, f64>,
}

impl RatesSnapshot {
    /// Age of this snapshot in hours from now.
    pub fn age_hours(&self) -> f64 {
        let fetched = self
            .fetched_at
            .parse::<DateTime<Utc>>()
            .unwrap_or(Utc::now());
        let elapsed = Utc::now().signed_duration_since(fetched);
        elapsed.num_seconds() as f64 / 3600.0
    }
}

// ── Frankfurter API response ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct FrankfurterResponse {
    base: String,
    rates: HashMap<String, f64>,
}

// ── Errors ──────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum CurrencyError {
    #[error("HTTP fetch failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Cache IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Cache parse error: {0}")]
    Parse(#[from] serde_json::Error),
}

// ── Cache path helpers ──────────────────────────────────────────────────────

/// Production cache path: ~/.cache/heimdall/fx.json
pub fn cache_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("heimdall")
        .join("fx.json")
}

// ── Cache read/write ────────────────────────────────────────────────────────

/// Read the cached rates snapshot from `path`.  Returns None if absent or
/// unparseable.
pub fn read_cache(path: &Path) -> Option<RatesSnapshot> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Write a rates snapshot to `path`, creating parent directories as needed.
pub fn write_cache(path: &Path, snapshot: &RatesSnapshot) -> Result<(), CurrencyError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec_pretty(snapshot)?;
    std::fs::write(path, json)?;
    Ok(())
}

// ── Live fetch ──────────────────────────────────────────────────────────────

/// Fetch fresh rates from Frankfurter.  Blocking (spawns a tiny runtime).
/// Returns None on any network or parse error.
fn fetch_live_rates() -> Option<RatesSnapshot> {
    fetch_from_url(FRANKFURTER_URL)
}

/// Fetch from an arbitrary URL — used internally and for testing.
fn fetch_from_url(url: &str) -> Option<RatesSnapshot> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .ok()?;
    rt.block_on(async {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(FETCH_TIMEOUT_SECS))
            .build()
            .ok()?;
        let resp = client.get(url).send().await.ok()?;
        let body: FrankfurterResponse = resp.json().await.ok()?;
        Some(RatesSnapshot {
            fetched_at: Utc::now().to_rfc3339(),
            base: body.base,
            rates: body.rates,
        })
    })
}

// ── Core public API ─────────────────────────────────────────────────────────

/// Convert `amount_usd` to `target` currency.
///
/// Resolution order:
/// 1. USD short-circuit (returns immediately, source = Live)
/// 2. Try live fetch; write to cache if successful
/// 3. Try existing on-disk cache
/// 4. Fallback: return amount unchanged as USD
pub fn convert_from_usd(amount_usd: f64, target: &str) -> ConvertResult {
    convert_with_snapshot(amount_usd, target, None, true)
}

/// Internal conversion that accepts an injected snapshot and an optional fetch
/// flag — used by tests to avoid network calls.
#[allow(clippy::collapsible_if)]
pub fn convert_with_snapshot(
    amount_usd: f64,
    target: &str,
    injected: Option<&RatesSnapshot>,
    allow_live_fetch: bool,
) -> ConvertResult {
    // USD short-circuit
    if target.eq_ignore_ascii_case("USD") {
        return ConvertResult {
            amount: amount_usd,
            currency: "USD".into(),
            source: RateSource::Live,
        };
    }

    // Normalise target to uppercase
    let target_upper = target.to_ascii_uppercase();

    // Use injected snapshot when provided (tests)
    if let Some(snap) = injected {
        return apply_rate(
            amount_usd,
            &target_upper,
            snap,
            RateSource::Cached { age_hours: 0.0 },
        );
    }

    // Attempt live fetch
    if allow_live_fetch {
        if let Some(snap) = fetch_live_rates() {
            let path = cache_path();
            let _ = write_cache(&path, &snap);
            let result = apply_rate(amount_usd, &target_upper, &snap, RateSource::Live);
            if !matches!(result.source, RateSource::Fallback) {
                return result;
            }
        }
    }

    // Try on-disk cache
    let path = cache_path();
    if let Some(snap) = read_cache(&path) {
        let age = snap.age_hours();
        let result = apply_rate(
            amount_usd,
            &target_upper,
            &snap,
            RateSource::Cached { age_hours: age },
        );
        if !matches!(result.source, RateSource::Fallback) {
            return result;
        }
    }

    // Ultimate fallback
    ConvertResult {
        amount: amount_usd,
        currency: "USD".into(),
        source: RateSource::Fallback,
    }
}

/// Apply a rate from `snapshot` for `target`.  Returns Fallback if target
/// not present in snapshot.
fn apply_rate(
    amount_usd: f64,
    target: &str,
    snap: &RatesSnapshot,
    source: RateSource,
) -> ConvertResult {
    match snap.rates.get(target) {
        Some(&rate) => ConvertResult {
            amount: (amount_usd * rate * 10000.0).round() / 10000.0,
            currency: target.to_string(),
            source,
        },
        None => ConvertResult {
            amount: amount_usd,
            currency: "USD".into(),
            source: RateSource::Fallback,
        },
    }
}

/// Small hardcoded set used when no cache exists yet.  Currently consumed
/// only by the test that locks in major-currency coverage; gated `#[cfg(test)]`
/// to keep release builds free of dead-code warnings.  Restore as a public
/// fallback when a non-test caller materialises.
#[cfg(test)]
fn hardcoded_currencies() -> Vec<String> {
    let mut v: Vec<String> = [
        "AUD", "BRL", "CAD", "CHF", "CNY", "CZK", "DKK", "EUR", "GBP", "HKD", "HUF", "IDR", "ILS",
        "INR", "ISK", "JPY", "KRW", "MXN", "MYR", "NOK", "NZD", "PHP", "PLN", "RON", "SEK", "SGD",
        "THB", "TRY", "ZAR",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    v.sort();
    v
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Test-only helper: build cache path relative to an explicit home dir.
    fn cache_path_for(home: &Path) -> PathBuf {
        home.join(".cache").join("heimdall").join("fx.json")
    }

    fn make_snapshot(rates: &[(&str, f64)]) -> RatesSnapshot {
        let mut map = HashMap::new();
        for (code, rate) in rates {
            map.insert(code.to_string(), *rate);
        }
        RatesSnapshot {
            fetched_at: Utc::now().to_rfc3339(),
            base: "USD".into(),
            rates: map,
        }
    }

    // ── USD short-circuit ────────────────────────────────────────────────

    #[test]
    fn test_usd_short_circuit_returns_live() {
        let result = convert_with_snapshot(42.0, "USD", None, false);
        assert_eq!(result.currency, "USD");
        assert!((result.amount - 42.0).abs() < 1e-9);
        assert!(matches!(result.source, RateSource::Live));
    }

    #[test]
    fn test_usd_short_circuit_lowercase() {
        let result = convert_with_snapshot(10.0, "usd", None, false);
        assert_eq!(result.currency, "USD");
        assert!(matches!(result.source, RateSource::Live));
    }

    // ── Cache round-trip ─────────────────────────────────────────────────

    #[test]
    fn test_cache_round_trip() {
        let tmp = TempDir::new().unwrap();
        let path = cache_path_for(tmp.path());
        let snap = make_snapshot(&[("EUR", 0.93), ("GBP", 0.78)]);
        write_cache(&path, &snap).unwrap();
        let loaded = read_cache(&path).unwrap();
        assert_eq!(loaded.base, "USD");
        assert!((loaded.rates["EUR"] - 0.93).abs() < 1e-9);
        assert!((loaded.rates["GBP"] - 0.78).abs() < 1e-9);
    }

    #[test]
    fn test_cache_creates_parent_dirs() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("a").join("b").join("fx.json");
        let snap = make_snapshot(&[("EUR", 0.9)]);
        write_cache(&path, &snap).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_read_cache_missing_returns_none() {
        let tmp = TempDir::new().unwrap();
        let path = cache_path_for(tmp.path());
        assert!(read_cache(&path).is_none());
    }

    // ── Conversion with injected snapshot ───────────────────────────────

    #[test]
    fn test_convert_eur_with_injected_snapshot() {
        let snap = make_snapshot(&[("EUR", 0.93)]);
        let result = convert_with_snapshot(100.0, "EUR", Some(&snap), false);
        assert_eq!(result.currency, "EUR");
        assert!((result.amount - 93.0).abs() < 0.01);
        // Injected always reports Cached
        assert!(matches!(result.source, RateSource::Cached { .. }));
    }

    #[test]
    fn test_convert_zero_amount() {
        let snap = make_snapshot(&[("EUR", 0.93)]);
        let result = convert_with_snapshot(0.0, "EUR", Some(&snap), false);
        assert!((result.amount - 0.0).abs() < 1e-9);
        assert_eq!(result.currency, "EUR");
    }

    // ── Fallback scenarios ───────────────────────────────────────────────

    #[test]
    fn test_fallback_when_no_cache_and_no_network() {
        // no injected snapshot, no live fetch allowed
        let result = convert_with_snapshot(50.0, "EUR", None, false);
        assert_eq!(result.currency, "USD");
        assert!((result.amount - 50.0).abs() < 1e-9);
        assert!(matches!(result.source, RateSource::Fallback));
    }

    #[test]
    fn test_fallback_when_target_missing_from_snapshot() {
        let snap = make_snapshot(&[("EUR", 0.93)]);
        let result = convert_with_snapshot(42.0, "XYZ", Some(&snap), false);
        assert_eq!(result.currency, "USD");
        assert!((result.amount - 42.0).abs() < 1e-9);
        assert!(matches!(result.source, RateSource::Fallback));
    }

    // ── hardcoded_currencies fallback ────────────────────────────────────

    #[test]
    fn test_hardcoded_currencies_contains_major() {
        let currencies = hardcoded_currencies();
        assert!(currencies.contains(&"EUR".to_string()));
        assert!(currencies.contains(&"GBP".to_string()));
        assert!(currencies.contains(&"JPY".to_string()));
    }

    // ── RateSource helpers ───────────────────────────────────────────────

    #[test]
    fn test_rate_source_as_str() {
        assert_eq!(RateSource::Live.as_str(), "live");
        assert_eq!(RateSource::Cached { age_hours: 3.5 }.as_str(), "cached");
        assert_eq!(RateSource::Fallback.as_str(), "fallback");
    }

    #[test]
    fn test_rate_source_age_hours() {
        assert_eq!(RateSource::Live.age_hours(), None);
        assert_eq!(RateSource::Fallback.age_hours(), None);
        assert!((RateSource::Cached { age_hours: 3.5 }.age_hours().unwrap() - 3.5).abs() < 1e-9);
    }

    // ── Case normalisation ───────────────────────────────────────────────

    #[test]
    fn test_target_case_insensitive() {
        let snap = make_snapshot(&[("EUR", 0.93)]);
        let r1 = convert_with_snapshot(100.0, "eur", Some(&snap), false);
        let r2 = convert_with_snapshot(100.0, "EUR", Some(&snap), false);
        assert_eq!(r1.currency, r2.currency);
        assert!((r1.amount - r2.amount).abs() < 1e-9);
    }
}

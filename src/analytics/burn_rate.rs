use serde::Serialize;

/// Burn-rate tier — maps tokens/min to a named severity band.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BurnRateTier {
    Normal,
    Moderate,
    High,
}

/// Thresholds used to map tokens/min → `BurnRateTier`.
#[derive(Debug, Clone, Copy)]
pub struct BurnRateConfig {
    /// tokens/min at or below this value → Normal.
    pub normal_max: f64,
    /// tokens/min at or below this value (and above normal_max) → Moderate.
    pub moderate_max: f64,
}

impl Default for BurnRateConfig {
    fn default() -> Self {
        Self {
            normal_max: 4000.0,
            moderate_max: 10000.0,
        }
    }
}

impl BurnRateConfig {
    /// Build a config from raw statusline thresholds (`burn_rate_normal_max`
    /// and `burn_rate_moderate_max` in `[statusline]` TOML).
    pub fn from_thresholds(normal_max: f64, moderate_max: f64) -> Self {
        Self {
            normal_max,
            moderate_max,
        }
    }
}

/// Classify a burn rate (tokens/min) into a tier.
///
/// - NaN or non-finite → Normal
/// - negative → Normal
/// - `<= normal_max` → Normal
/// - `<= moderate_max` → Moderate
/// - `> moderate_max` → High
pub fn tier(tokens_per_min: f64, cfg: &BurnRateConfig) -> BurnRateTier {
    if !tokens_per_min.is_finite() || tokens_per_min <= cfg.normal_max {
        BurnRateTier::Normal
    } else if tokens_per_min <= cfg.moderate_max {
        BurnRateTier::Moderate
    } else {
        BurnRateTier::High
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default() -> BurnRateConfig {
        BurnRateConfig::default()
    }

    #[test]
    fn exactly_normal_max_is_normal() {
        assert_eq!(tier(4000.0, &default()), BurnRateTier::Normal);
    }

    #[test]
    fn just_above_normal_max_is_moderate() {
        assert_eq!(tier(4001.0, &default()), BurnRateTier::Moderate);
    }

    #[test]
    fn exactly_moderate_max_is_moderate() {
        assert_eq!(tier(10000.0, &default()), BurnRateTier::Moderate);
    }

    #[test]
    fn just_above_moderate_max_is_high() {
        assert_eq!(tier(10001.0, &default()), BurnRateTier::High);
    }

    #[test]
    fn nan_is_normal() {
        assert_eq!(tier(f64::NAN, &default()), BurnRateTier::Normal);
    }

    #[test]
    fn negative_is_normal() {
        assert_eq!(tier(-1.0, &default()), BurnRateTier::Normal);
    }

    #[test]
    fn threshold_override_changes_classification() {
        let cfg = BurnRateConfig {
            normal_max: 100.0,
            moderate_max: 500.0,
        };
        // 150 is Moderate under cfg but would be Normal under default.
        assert_eq!(tier(150.0, &cfg), BurnRateTier::Moderate);
        // 600 is High under cfg but Moderate under default.
        assert_eq!(tier(600.0, &cfg), BurnRateTier::High);
    }
}

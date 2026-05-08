use std::sync::LazyLock;

use regex::Regex;

use super::RedactedValue;

static SECRET_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(token|key|secret|password|auth|bearer|credential|api[_-]?key)")
        .expect("SECRET_RE is valid")
});

/// Redact an environment variable value if the key looks like a secret.
pub fn redact_env_value(key: &str, value: &str) -> RedactedValue {
    if SECRET_RE.is_match(key) {
        let masked = if value.len() > 8 {
            format!("•••••{}", &value[value.len() - 4..])
        } else {
            "•••••".to_string()
        };
        RedactedValue::Secret { masked }
    } else {
        RedactedValue::Plain {
            value: value.to_string(),
        }
    }
}

/// Strip query string and fragment from a URL, keeping scheme+host+port+path.
pub fn redact_url_for_display(url: &str) -> String {
    // Simple approach: find '?' or '#' and cut there.
    let end = url.find('?').or_else(|| url.find('#')).unwrap_or(url.len());
    url[..end].to_string()
}

/// Redact secret-looking values in `--key=value` style args.
pub fn redact_args(args: &[String]) -> Vec<String> {
    args.iter()
        .map(|arg| {
            if let Some((k, _v)) = arg.strip_prefix("--").and_then(|s| {
                let idx = s.find('=')?;
                Some((&s[..idx], &s[idx + 1..]))
            }) {
                if SECRET_RE.is_match(k) {
                    return format!("--{}=[REDACTED]", k);
                }
            }
            arg.clone()
        })
        .collect()
}

/// If this arg (or the next) is an env-file path, return the path.
/// Handles `--env-file=<path>` form (single arg).
pub fn is_env_file_arg(arg: &str) -> Option<String> {
    if let Some(path) = arg.strip_prefix("--env-file=") {
        return Some(path.to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_keys_are_masked() {
        let cases = [
            "GITHUB_TOKEN",
            "OPENAI_API_KEY",
            "auth_password",
            "BearerToken",
            "api-key",
            "API_KEY",
            "secret",
            "credential",
        ];
        for key in &cases {
            let r = redact_env_value(key, "supersecretvalue123");
            assert!(
                matches!(r, RedactedValue::Secret { .. }),
                "{key} should be secret"
            );
        }
    }

    #[test]
    fn non_secret_keys_are_plain() {
        let cases = ["HOST", "PORT", "LOG_LEVEL", "DEBUG", "TIMEOUT_MS"];
        for key in &cases {
            let r = redact_env_value(key, "somevalue");
            assert!(
                matches!(r, RedactedValue::Plain { .. }),
                "{key} should be plain"
            );
        }
    }

    #[test]
    fn short_secret_masked_without_suffix() {
        let r = redact_env_value("TOKEN", "abc");
        assert!(matches!(r, RedactedValue::Secret { ref masked } if masked == "•••••"));
    }

    #[test]
    fn long_secret_shows_last_four() {
        let r = redact_env_value("TOKEN", "supersecret1234");
        if let RedactedValue::Secret { masked } = r {
            assert!(masked.ends_with("1234"), "masked={masked}");
        } else {
            panic!("expected Secret");
        }
    }

    #[test]
    fn url_stripping_removes_query() {
        let url = "https://api.example.com:8080/v1/chat?key=secret&foo=bar";
        let r = redact_url_for_display(url);
        assert_eq!(r, "https://api.example.com:8080/v1/chat");
    }

    #[test]
    fn url_stripping_removes_fragment() {
        let url = "http://localhost/path#section";
        let r = redact_url_for_display(url);
        assert_eq!(r, "http://localhost/path");
    }

    #[test]
    fn url_no_query_unchanged() {
        let url = "https://api.example.com/v1";
        let r = redact_url_for_display(url);
        assert_eq!(r, url);
    }

    #[test]
    fn arg_redaction_replaces_secret_value() {
        let args = vec![
            "--api-key=mysecret".to_string(),
            "--host=localhost".to_string(),
        ];
        let r = redact_args(&args);
        assert_eq!(r[0], "--api-key=[REDACTED]");
        assert_eq!(r[1], "--host=localhost");
    }

    #[test]
    fn arg_redaction_plain_args_unchanged() {
        let args = vec!["--port=8080".to_string(), "--verbose".to_string()];
        let r = redact_args(&args);
        assert_eq!(r[0], "--port=8080");
        assert_eq!(r[1], "--verbose");
    }

    #[test]
    fn env_file_arg_parsed() {
        assert_eq!(
            is_env_file_arg("--env-file=/path/to/.env"),
            Some("/path/to/.env".to_string())
        );
        assert_eq!(is_env_file_arg("--other"), None);
    }
}

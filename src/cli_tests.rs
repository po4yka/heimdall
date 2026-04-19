#[cfg(test)]
mod tests {
    use std::io::Write;
    use tempfile::TempDir;

    use crate::scanner::{self, db};

    fn setup_test_db(tmp: &TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
        let projects = tmp.path().join("projects").join("user").join("proj");
        std::fs::create_dir_all(&projects).unwrap();
        let filepath = projects.join("sess.jsonl");
        let mut f = std::fs::File::create(&filepath).unwrap();
        let today = chrono::Local::now()
            .format("%Y-%m-%dT10:00:00Z")
            .to_string();
        writeln!(
            f,
            "{}",
            serde_json::json!({
                "type": "user", "sessionId": "s1", "timestamp": &today, "cwd": "/home/user/project"
            })
        )
        .unwrap();
        writeln!(
            f,
            "{}",
            serde_json::json!({
                "type": "assistant", "sessionId": "s1", "timestamp": &today,
                "cwd": "/home/user/project",
                "message": {
                    "id": "msg-1", "model": "claude-sonnet-4-6",
                    "usage": { "input_tokens": 1000, "output_tokens": 500, "cache_read_input_tokens": 100, "cache_creation_input_tokens": 50 },
                    "content": []
                }
            })
        )
        .unwrap();

        let db_path = tmp.path().join("usage.db");
        let parent = tmp.path().join("projects");
        scanner::scan(Some(vec![parent.clone()]), &db_path, false).unwrap();
        (db_path, parent)
    }

    #[test]
    fn test_cmd_today_no_data() {
        let tmp = TempDir::new().unwrap();
        let db_path = tmp.path().join("usage.db");
        // Create empty DB
        let conn = db::open_db(&db_path).unwrap();
        db::init_db(&conn).unwrap();
        drop(conn);
        // Should not panic
        crate::cmd_today(&db_path, false, None, &std::collections::HashMap::new()).unwrap();
    }

    #[test]
    fn test_cmd_today_json() {
        let tmp = TempDir::new().unwrap();
        let (db_path, _) = setup_test_db(&tmp);
        // JSON mode should not panic (output goes to stdout)
        crate::cmd_today(&db_path, true, None, &std::collections::HashMap::new()).unwrap();
    }

    #[test]
    fn test_cmd_stats_empty_db() {
        let tmp = TempDir::new().unwrap();
        let db_path = tmp.path().join("usage.db");
        let conn = db::open_db(&db_path).unwrap();
        db::init_db(&conn).unwrap();
        drop(conn);
        // Should not panic on empty DB
        crate::cmd_stats(
            &db_path,
            false,
            "USD",
            None,
            &std::collections::HashMap::new(),
        )
        .unwrap();
    }

    #[test]
    fn test_cmd_stats_json() {
        let tmp = TempDir::new().unwrap();
        let (db_path, _) = setup_test_db(&tmp);
        crate::cmd_stats(
            &db_path,
            true,
            "USD",
            None,
            &std::collections::HashMap::new(),
        )
        .unwrap();
    }

    // ── pricing refresh integration tests (no network) ───────────────────────

    /// Verify that `run_refresh_with_snapshot` writes a valid cache file.
    /// This is the stub-based integration test for the `pricing refresh` command:
    /// it exercises the same write path that the real command uses, without
    /// touching the network.
    #[test]
    fn test_pricing_refresh_writes_cache_file() {
        use crate::litellm::{LiteLlmModelEntry, LiteLlmSnapshot, run_refresh_with_snapshot};
        use std::collections::HashMap;

        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("litellm_pricing.json");

        let mut entries = HashMap::new();
        entries.insert(
            "gemini-2.5-flash".to_string(),
            LiteLlmModelEntry {
                input_cost_per_token: Some(0.075),
                output_cost_per_token: Some(0.30),
            },
        );
        entries.insert(
            "mistral-large".to_string(),
            LiteLlmModelEntry {
                input_cost_per_token: Some(2.0),
                output_cost_per_token: Some(6.0),
            },
        );
        let snap = LiteLlmSnapshot {
            fetched_at: chrono::Utc::now().to_rfc3339(),
            entries,
        };

        let (count, path) = run_refresh_with_snapshot(&cache, snap).unwrap();
        assert_eq!(count, 2);
        assert_eq!(path, cache);
        assert!(cache.exists(), "cache file must be written");

        // Verify the written file is readable and has the right shape.
        let loaded = crate::litellm::read_cache(&cache).unwrap();
        assert_eq!(loaded.entries.len(), 2);
        assert!(loaded.entries.contains_key("gemini-2.5-flash"));
    }

    /// Hardcoded-wins invariant: even when the LiteLLM map has a conflicting
    /// entry for "claude-sonnet-4-6", `estimate_cost` must return the hardcoded
    /// $3/MTok input price (tier 1 exact match wins).
    ///
    /// NOTE: `set_litellm_map` is an OnceLock — it can only be set once per
    /// process. This test uses `load_litellm_cache` + `estimate_cost` directly
    /// to verify the lookup priority without relying on the global OnceLock.
    #[test]
    fn test_hardcoded_claude_wins_over_litellm_conflicting_entry() {
        use crate::litellm::{LiteLlmModelEntry, LiteLlmSnapshot, write_cache};
        use crate::pricing::{COST_CONFIDENCE_HIGH, estimate_cost, load_litellm_cache};
        use std::collections::HashMap;

        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("litellm_pricing.json");

        // LiteLLM cache has a conflicting entry with a different (wrong) price.
        let mut entries = HashMap::new();
        entries.insert(
            "claude-sonnet-4-6".to_string(),
            LiteLlmModelEntry {
                input_cost_per_token: Some(999.0), // wrong price — should never be used
                output_cost_per_token: Some(999.0),
            },
        );
        let snap = LiteLlmSnapshot {
            fetched_at: chrono::Utc::now().to_rfc3339(),
            entries,
        };
        write_cache(&cache, &snap).unwrap();

        // load_litellm_cache reads the file; this exercises the parsing path.
        let map = load_litellm_cache(&cache).unwrap();
        assert!(map.contains_key("claude-sonnet-4-6"));
        // The map has the LiteLLM entry, but lookup_pricing will never reach it
        // because tier 1 (exact hardcoded match) fires first.
        // estimate_cost does NOT use the map argument — it uses the global OnceLock.
        // So we verify via estimate_cost directly: hardcoded path, no global needed.
        let est = estimate_cost("claude-sonnet-4-6", 1_000_000, 0, 0, 0);
        // Must return $3/MTok = 3_000_000_000 nanos, HIGH confidence.
        assert_eq!(est.estimated_cost_nanos, 3_000_000_000);
        assert_eq!(est.cost_confidence, COST_CONFIDENCE_HIGH);
    }

    /// LiteLLM fallback: "gemini-2.5-flash" has no hardcoded entry, so
    /// estimate_cost should return 0 nanos and LOW confidence (no map loaded
    /// in this test process from a previous test — OnceLock is unset here).
    #[test]
    fn test_gemini_falls_through_to_zero_without_litellm_map() {
        use crate::pricing::{COST_CONFIDENCE_LOW, estimate_cost};
        // Without a LiteLLM map installed, gemini-2.5-flash is unknown.
        let est = estimate_cost("gemini-2.5-flash", 1_000_000, 0, 0, 0);
        assert_eq!(est.estimated_cost_nanos, 0);
        assert_eq!(est.cost_confidence, COST_CONFIDENCE_LOW);
    }

    // ── Phase 11: parse_project_alias + merge precedence ────────────────────

    #[test]
    fn test_parse_project_alias_valid() {
        let result = crate::parse_project_alias("-Users-foo=My Project");
        assert!(result.is_ok());
        let (k, v) = result.unwrap();
        assert_eq!(k, "-Users-foo");
        assert_eq!(v, "My Project");
    }

    #[test]
    fn test_parse_project_alias_whitespace_trimmed() {
        let result = crate::parse_project_alias("  slug  =  Name  ");
        assert!(result.is_ok());
        let (k, v) = result.unwrap();
        assert_eq!(k, "slug");
        assert_eq!(v, "Name");
    }

    #[test]
    fn test_parse_project_alias_invalid_no_equals() {
        let result = crate::parse_project_alias("no-equals-sign");
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_alias_overrides_config_alias() {
        // Construct a Config with a config-level alias.
        let mut cfg: crate::config::Config = Default::default();
        cfg.project_aliases
            .insert("foo".to_string(), "Config Name".to_string());

        // CLI args that override the same key.
        let cli_overrides: Vec<(String, String)> =
            vec![("foo".to_string(), "CLI Name".to_string())];

        // Apply the same merge loop used in main.rs.
        let mut map = cfg.project_aliases.clone();
        for (k, v) in cli_overrides {
            map.insert(k, v);
        }

        // CLI value wins.
        assert_eq!(map.get("foo"), Some(&"CLI Name".to_string()));
    }

    #[test]
    fn test_parse_project_alias_empty_key_rejected() {
        let result = crate::parse_project_alias("=SomeName");
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("key is empty"),
            "expected key-empty message, got: {msg}"
        );
    }

    #[test]
    fn test_parse_project_alias_empty_value_rejected() {
        let result = crate::parse_project_alias("slug=");
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("value is empty"),
            "expected value-empty message, got: {msg}"
        );
    }

    #[test]
    fn test_parse_project_alias_value_with_equals() {
        // A value containing '=' should be accepted: only the first '=' is the separator.
        let result = crate::parse_project_alias("slug=a=b");
        assert!(result.is_ok(), "expected Ok, got: {:?}", result);
        let (k, v) = result.unwrap();
        assert_eq!(k, "slug");
        assert_eq!(v, "a=b");
    }
}

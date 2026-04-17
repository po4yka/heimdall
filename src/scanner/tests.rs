//! Integration tests for the scan pipeline.

#[cfg(test)]
mod tests {
    use std::io::Write;
    use tempfile::TempDir;

    use crate::scanner;
    use crate::scanner::db;

    fn make_assistant(session_id: &str, ts: &str, input: i64, output: i64, msg_id: &str) -> String {
        let mut msg = serde_json::json!({
            "model": "claude-sonnet-4-6",
            "usage": {
                "input_tokens": input,
                "output_tokens": output,
                "cache_read_input_tokens": 0,
                "cache_creation_input_tokens": 0,
            },
            "content": [],
        });
        if !msg_id.is_empty() {
            msg["id"] = serde_json::json!(msg_id);
        }
        serde_json::json!({
            "type": "assistant",
            "sessionId": session_id,
            "timestamp": ts,
            "cwd": "/home/user/project",
            "message": msg,
        })
        .to_string()
    }

    fn make_user(session_id: &str, ts: &str) -> String {
        serde_json::json!({
            "type": "user",
            "sessionId": session_id,
            "timestamp": ts,
            "cwd": "/home/user/project",
        })
        .to_string()
    }

    fn make_custom_title(session_id: &str, title: &str) -> String {
        serde_json::json!({
            "type": "custom-title",
            "sessionId": session_id,
            "customTitle": title,
        })
        .to_string()
    }

    fn make_assistant_with_tools(
        session_id: &str,
        ts: &str,
        msg_id: &str,
        tools: &[(&str, &str)],
    ) -> String {
        let content: Vec<serde_json::Value> = tools
            .iter()
            .map(|(tool_use_id, tool_name)| {
                serde_json::json!({
                    "type": "tool_use",
                    "id": tool_use_id,
                    "name": tool_name,
                    "input": {},
                })
            })
            .collect();

        serde_json::json!({
            "type": "assistant",
            "sessionId": session_id,
            "timestamp": ts,
            "cwd": "/home/user/project",
            "message": {
                "id": msg_id,
                "model": "claude-sonnet-4-6",
                "usage": {
                    "input_tokens": 100,
                    "output_tokens": 50,
                    "cache_read_input_tokens": 0,
                    "cache_creation_input_tokens": 0,
                },
                "content": content,
            }
        })
        .to_string()
    }

    fn write_project_jsonl(
        projects_dir: &std::path::Path,
        project: &str,
        filename: &str,
        lines: &[String],
    ) -> std::path::PathBuf {
        let dir = projects_dir.join(project);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(filename);
        let mut f = std::fs::File::create(&path).unwrap();
        for line in lines {
            writeln!(f, "{}", line).unwrap();
        }
        path
    }

    #[test]
    fn test_scan_new_files() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path().join("projects");
        let db_path = tmp.path().join("usage.db");

        write_project_jsonl(
            &projects,
            "user/myproj",
            "sess-1.jsonl",
            &[
                make_user("s1", "2026-04-08T09:00:00Z"),
                make_assistant("s1", "2026-04-08T09:01:00Z", 100, 50, "msg-1"),
                make_assistant("s1", "2026-04-08T09:02:00Z", 200, 100, "msg-2"),
            ],
        );

        let result = scanner::scan(Some(vec![projects.clone()]), &db_path, false).unwrap();
        assert_eq!(result.new, 1);
        assert_eq!(result.turns, 2);
        assert!(result.sessions > 0);

        // Verify DB contents
        let conn = db::open_db(&db_path).unwrap();
        let turn_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM turns", [], |r| r.get(0))
            .unwrap();
        assert_eq!(turn_count, 2);

        let session: (i64, i64, i64) = conn
            .query_row(
                "SELECT total_input_tokens, total_output_tokens, turn_count FROM sessions WHERE session_id = 'claude:s1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(session, (300, 150, 2));
    }

    #[test]
    fn test_scan_incremental_skip() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path().join("projects");
        let db_path = tmp.path().join("usage.db");

        write_project_jsonl(
            &projects,
            "user/proj",
            "sess-1.jsonl",
            &[
                make_user("s1", "2026-04-08T09:00:00Z"),
                make_assistant("s1", "2026-04-08T09:01:00Z", 100, 50, "msg-1"),
            ],
        );

        scanner::scan(Some(vec![projects.clone()]), &db_path, false).unwrap();

        // Second scan: same file, should skip
        let result = scanner::scan(Some(vec![projects.clone()]), &db_path, false).unwrap();
        assert_eq!(result.skipped, 1);
        assert_eq!(result.new, 0);
        assert_eq!(result.updated, 0);
    }

    #[test]
    fn test_scan_incremental_update() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path().join("projects");
        let db_path = tmp.path().join("usage.db");

        let filepath = write_project_jsonl(
            &projects,
            "user/proj",
            "sess-1.jsonl",
            &[
                make_user("s1", "2026-04-08T09:00:00Z"),
                make_assistant("s1", "2026-04-08T09:01:00Z", 100, 50, "msg-1"),
            ],
        );

        scanner::scan(Some(vec![projects.clone()]), &db_path, false).unwrap();

        // Append new lines and bump mtime
        std::thread::sleep(std::time::Duration::from_millis(50));
        {
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(&filepath)
                .unwrap();
            writeln!(
                f,
                "{}",
                make_assistant("s1", "2026-04-08T09:05:00Z", 200, 100, "msg-2")
            )
            .unwrap();
        }

        let result = scanner::scan(Some(vec![projects.clone()]), &db_path, false).unwrap();
        assert_eq!(result.updated, 1);
        assert_eq!(result.turns, 2); // changed files are reparsed from scratch

        let conn = db::open_db(&db_path).unwrap();
        let total_turns: i64 = conn
            .query_row("SELECT COUNT(*) FROM turns", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total_turns, 2);

        let (total_in, total_out): (i64, i64) = conn
            .query_row(
                "SELECT total_input_tokens, total_output_tokens FROM sessions WHERE session_id = 'claude:s1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(total_in, 300);
        assert_eq!(total_out, 150);
    }

    #[test]
    fn test_scan_rewritten_file_replaces_old_turns() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path().join("projects");
        let db_path = tmp.path().join("usage.db");

        let filepath = write_project_jsonl(
            &projects,
            "user/proj",
            "sess-1.jsonl",
            &[
                make_user("s1", "2026-04-08T09:00:00Z"),
                make_assistant("s1", "2026-04-08T09:01:00Z", 100, 50, "msg-1"),
                make_assistant("s1", "2026-04-08T09:02:00Z", 200, 100, "msg-2"),
            ],
        );

        scanner::scan(Some(vec![projects.clone()]), &db_path, false).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(50));
        let mut f = std::fs::File::create(&filepath).unwrap();
        writeln!(f, "{}", make_user("s1", "2026-04-08T09:00:00Z")).unwrap();
        writeln!(
            f,
            "{}",
            make_assistant("s1", "2026-04-08T09:10:00Z", 300, 150, "msg-3")
        )
        .unwrap();

        let result = scanner::scan(Some(vec![projects.clone()]), &db_path, false).unwrap();
        assert_eq!(result.updated, 1);
        assert_eq!(result.turns, 1);

        let conn = db::open_db(&db_path).unwrap();
        let (turn_count, total_in, total_out): (i64, i64, i64) = conn
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0) FROM turns",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(turn_count, 1);
        assert_eq!(total_in, 300);
        assert_eq!(total_out, 150);
    }

    #[test]
    fn test_scan_truncated_file_removes_stale_turns() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path().join("projects");
        let db_path = tmp.path().join("usage.db");

        let filepath = write_project_jsonl(
            &projects,
            "user/proj",
            "sess-1.jsonl",
            &[
                make_user("s1", "2026-04-08T09:00:00Z"),
                make_assistant("s1", "2026-04-08T09:01:00Z", 100, 50, "msg-1"),
                make_assistant("s1", "2026-04-08T09:02:00Z", 200, 100, "msg-2"),
            ],
        );

        scanner::scan(Some(vec![projects.clone()]), &db_path, false).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(50));
        let mut f = std::fs::File::create(&filepath).unwrap();
        writeln!(f, "{}", make_user("s1", "2026-04-08T09:00:00Z")).unwrap();

        let result = scanner::scan(Some(vec![projects.clone()]), &db_path, false).unwrap();
        assert_eq!(result.updated, 1);
        assert_eq!(result.turns, 0);

        let conn = db::open_db(&db_path).unwrap();
        let (turn_count, session_turn_count): (i64, i64) = conn
            .query_row(
                "SELECT
                    (SELECT COUNT(*) FROM turns),
                    COALESCE((SELECT turn_count FROM sessions WHERE session_id = 'claude:s1'), 0)",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(turn_count, 0);
        assert_eq!(session_turn_count, 0);
    }

    #[test]
    fn test_scan_mixed_model_session_uses_latest_model() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path().join("projects");
        let db_path = tmp.path().join("usage.db");

        write_project_jsonl(
            &projects,
            "user/proj",
            "sess-1.jsonl",
            &[
                make_user("s1", "2026-04-08T09:00:00Z"),
                make_assistant("s1", "2026-04-08T09:01:00Z", 100, 50, "msg-1"),
                serde_json::json!({
                    "type": "assistant",
                    "sessionId": "s1",
                    "timestamp": "2026-04-08T09:10:00Z",
                    "cwd": "/home/user/project",
                    "message": {
                        "id": "msg-2",
                        "model": "claude-opus-4-6",
                        "usage": {
                            "input_tokens": 120,
                            "output_tokens": 60,
                            "cache_read_input_tokens": 0,
                            "cache_creation_input_tokens": 0
                        },
                        "content": []
                    }
                })
                .to_string(),
            ],
        );

        scanner::scan(Some(vec![projects.clone()]), &db_path, false).unwrap();

        let conn = db::open_db(&db_path).unwrap();
        let session_model: Option<String> = conn
            .query_row(
                "SELECT model FROM sessions WHERE session_id = 'claude:s1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(session_model.as_deref(), Some("claude-opus-4-6"));
    }

    #[test]
    fn test_scan_rewritten_file_updates_and_clears_session_title() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path().join("projects");
        let db_path = tmp.path().join("usage.db");

        let filepath = write_project_jsonl(
            &projects,
            "user/proj",
            "sess-1.jsonl",
            &[
                make_custom_title("s1", "Initial title"),
                make_user("s1", "2026-04-08T09:00:00Z"),
                make_assistant("s1", "2026-04-08T09:01:00Z", 100, 50, "msg-1"),
            ],
        );

        scanner::scan(Some(vec![projects.clone()]), &db_path, false).unwrap();

        let conn = db::open_db(&db_path).unwrap();
        let title: Option<String> = conn
            .query_row(
                "SELECT title FROM sessions WHERE session_id = 'claude:s1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(title.as_deref(), Some("Initial title"));
        drop(conn);

        std::thread::sleep(std::time::Duration::from_millis(50));
        let mut f = std::fs::File::create(&filepath).unwrap();
        writeln!(f, "{}", make_custom_title("s1", "Renamed title")).unwrap();
        writeln!(f, "{}", make_user("s1", "2026-04-08T09:00:00Z")).unwrap();
        writeln!(
            f,
            "{}",
            make_assistant("s1", "2026-04-08T09:01:00Z", 100, 50, "msg-1")
        )
        .unwrap();

        scanner::scan(Some(vec![projects.clone()]), &db_path, false).unwrap();

        let conn = db::open_db(&db_path).unwrap();
        let title: Option<String> = conn
            .query_row(
                "SELECT title FROM sessions WHERE session_id = 'claude:s1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(title.as_deref(), Some("Renamed title"));
        drop(conn);

        std::thread::sleep(std::time::Duration::from_millis(50));
        let mut f = std::fs::File::create(&filepath).unwrap();
        writeln!(f, "{}", make_user("s1", "2026-04-08T09:00:00Z")).unwrap();
        writeln!(
            f,
            "{}",
            make_assistant("s1", "2026-04-08T09:01:00Z", 100, 50, "msg-1")
        )
        .unwrap();

        scanner::scan(Some(vec![projects]), &db_path, false).unwrap();

        let conn = db::open_db(&db_path).unwrap();
        let title: Option<String> = conn
            .query_row(
                "SELECT title FROM sessions WHERE session_id = 'claude:s1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(title.is_none());
    }

    #[test]
    fn test_scan_rewritten_file_replaces_tool_invocations() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path().join("projects");
        let db_path = tmp.path().join("usage.db");

        let filepath = write_project_jsonl(
            &projects,
            "user/proj",
            "sess-1.jsonl",
            &[
                make_user("s1", "2026-04-08T09:00:00Z"),
                make_assistant_with_tools(
                    "s1",
                    "2026-04-08T09:01:00Z",
                    "msg-1",
                    &[("tool-1", "Read"), ("tool-2", "Read")],
                ),
            ],
        );

        scanner::scan(Some(vec![projects.clone()]), &db_path, false).unwrap();

        let conn = db::open_db(&db_path).unwrap();
        let tool_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tool_invocations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(tool_count, 2);
        drop(conn);

        std::thread::sleep(std::time::Duration::from_millis(50));
        let mut f = std::fs::File::create(&filepath).unwrap();
        writeln!(f, "{}", make_user("s1", "2026-04-08T09:00:00Z")).unwrap();
        writeln!(
            f,
            "{}",
            make_assistant("s1", "2026-04-08T09:05:00Z", 200, 100, "msg-2")
        )
        .unwrap();

        scanner::scan(Some(vec![projects]), &db_path, false).unwrap();

        let conn = db::open_db(&db_path).unwrap();
        let tool_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tool_invocations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(tool_count, 0);
    }

    #[test]
    fn test_scan_multiple_files() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path().join("projects");
        let db_path = tmp.path().join("usage.db");

        write_project_jsonl(
            &projects,
            "user/proj-a",
            "s1.jsonl",
            &[
                make_user("s1", "2026-04-08T09:00:00Z"),
                make_assistant("s1", "2026-04-08T09:01:00Z", 100, 50, "msg-1"),
            ],
        );
        write_project_jsonl(
            &projects,
            "user/proj-b",
            "s2.jsonl",
            &[
                make_user("s2", "2026-04-08T10:00:00Z"),
                make_assistant("s2", "2026-04-08T10:01:00Z", 200, 100, "msg-2"),
                make_assistant("s2", "2026-04-08T10:02:00Z", 300, 150, "msg-3"),
            ],
        );

        let result = scanner::scan(Some(vec![projects]), &db_path, false).unwrap();
        assert_eq!(result.new, 2);
        assert_eq!(result.turns, 3);
    }

    #[test]
    fn test_scan_streaming_dedup_across_files() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path().join("projects");
        let db_path = tmp.path().join("usage.db");

        // Same message_id in two files -- should only store once
        write_project_jsonl(
            &projects,
            "user/proj",
            "file1.jsonl",
            &[
                make_user("s1", "2026-04-08T09:00:00Z"),
                make_assistant("s1", "2026-04-08T09:01:00Z", 100, 50, "msg-dup"),
            ],
        );
        write_project_jsonl(
            &projects,
            "user/proj",
            "file2.jsonl",
            &[
                make_user("s1", "2026-04-08T09:00:00Z"),
                make_assistant("s1", "2026-04-08T09:01:00Z", 100, 50, "msg-dup"),
                make_assistant("s1", "2026-04-08T09:02:00Z", 200, 100, "msg-new"),
            ],
        );

        scanner::scan(Some(vec![projects]), &db_path, false).unwrap();

        let conn = db::open_db(&db_path).unwrap();
        let turn_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM turns", [], |r| r.get(0))
            .unwrap();
        assert_eq!(turn_count, 2); // msg-dup deduped, msg-new kept

        let (total_in, turn_count_session): (i64, i64) = conn
            .query_row(
                "SELECT total_input_tokens, turn_count FROM sessions WHERE session_id = 'claude:s1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(total_in, 300); // 100 + 200
        assert_eq!(turn_count_session, 2);
    }

    #[test]
    fn test_scan_empty_directory() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path().join("projects");
        std::fs::create_dir_all(&projects).unwrap();
        let db_path = tmp.path().join("usage.db");

        let result = scanner::scan(Some(vec![projects]), &db_path, false).unwrap();
        assert_eq!(result.new, 0);
        assert_eq!(result.turns, 0);
    }

    fn make_subagent_assistant(
        session_id: &str,
        agent_id: &str,
        input: i64,
        output: i64,
        msg_id: &str,
    ) -> String {
        let mut msg = serde_json::json!({
            "model": "claude-sonnet-4-6",
            "usage": {
                "input_tokens": input,
                "output_tokens": output,
                "cache_read_input_tokens": 0,
                "cache_creation_input_tokens": 0,
            },
            "content": [],
        });
        if !msg_id.is_empty() {
            msg["id"] = serde_json::json!(msg_id);
        }
        serde_json::json!({
            "type": "assistant",
            "sessionId": session_id,
            "timestamp": "2026-04-08T10:00:00Z",
            "cwd": "/home/user/project",
            "isSidechain": true,
            "agentId": agent_id,
            "message": msg,
        })
        .to_string()
    }

    #[test]
    fn test_scan_subagent_records() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path().join("projects");
        let db_path = tmp.path().join("usage.db");

        write_project_jsonl(
            &projects,
            "user/proj",
            "sess.jsonl",
            &[
                make_user("s1", "2026-04-08T09:00:00Z"),
                make_assistant("s1", "2026-04-08T09:01:00Z", 100, 50, "msg-p1"),
                make_subagent_assistant("s1", "agent-abc", 200, 100, "msg-a1"),
            ],
        );

        scanner::scan(Some(vec![projects]), &db_path, false).unwrap();

        let conn = db::open_db(&db_path).unwrap();
        let subagent_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM turns WHERE is_subagent = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(subagent_count, 1);

        let agent_id: Option<String> = conn
            .query_row(
                "SELECT agent_id FROM turns WHERE is_subagent = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(agent_id.unwrap(), "agent-abc");
    }

    #[test]
    fn test_dashboard_data_after_scan() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path().join("projects");
        let db_path = tmp.path().join("usage.db");

        write_project_jsonl(
            &projects,
            "user/proj",
            "s1.jsonl",
            &[
                make_user("s1", "2026-04-08T09:00:00Z"),
                make_assistant("s1", "2026-04-08T09:01:00Z", 1000, 500, "msg-1"),
            ],
        );

        scanner::scan(Some(vec![projects]), &db_path, false).unwrap();

        let conn = db::open_db(&db_path).unwrap();
        let data = db::get_dashboard_data(&conn).unwrap();

        assert!(!data.all_models.is_empty());
        assert!(data.all_models.contains(&"claude-sonnet-4-6".to_string()));
        assert_eq!(data.sessions_all.len(), 1);
        assert_eq!(data.sessions_all[0].input, 1000);
        assert_eq!(data.sessions_all[0].output, 500);
        assert!(!data.daily_by_model.is_empty());
        assert_eq!(data.daily_by_model[0].day, "2026-04-08");
    }

    #[test]
    fn registry_all_returns_at_least_two_providers() {
        let providers = crate::scanner::providers::all();
        assert!(providers.len() >= 2);
        let names: Vec<&str> = providers.iter().map(|p| p.name()).collect();
        assert!(names.contains(&"claude"));
        assert!(names.contains(&"codex"));
    }

    #[test]
    fn test_provider_column_backfill_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let db_path = tmp.path().join("usage.db");
        let conn = db::open_db(&db_path).unwrap();
        db::init_db(&conn).unwrap();
        // Second call must not error
        db::init_db(&conn).unwrap();
        // All sessions rows must have non-empty provider
        let bad: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sessions WHERE provider IS NULL OR provider = ''",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(bad, 0);
    }
}

//! Post-import ingestion: synthesise Turn + Session rows from parsed export
//! conversations and insert them into the main scanner SQLite database.
//!
//! Tokens are populated from the export metadata where available.  OpenAI's
//! public `conversations.json` format does not include token counts, so those
//! rows land with zero tokens and `cost_confidence = "none"`.

use anyhow::Result;
use rusqlite::Connection;

use crate::models::{Session, Turn};
use crate::scanner::db::{insert_turns, upsert_sessions};
use crate::scanner::parser::aggregate_sessions;

const PROVIDER_CLAUDE_EXPORT: &str = "claude_export";
const PROVIDER_CHATGPT_EXPORT: &str = "chatgpt_export";

fn epoch_to_rfc3339(epoch: f64) -> String {
    use std::time::{Duration, UNIX_EPOCH};
    let dur = Duration::from_secs_f64(epoch.max(0.0));
    let sys = UNIX_EPOCH + dur;
    let dt: chrono::DateTime<chrono::Utc> = sys.into();
    dt.to_rfc3339()
}

/// Ingest a single Anthropic `NormalizedConversation` into the scanner DB.
/// Returns the number of new turn rows inserted (0 if all already present).
pub fn ingest_anthropic(
    conn: &Connection,
    conv: &super::anthropic::NormalizedConversation,
) -> Result<usize> {
    let session_id = format!("{}:{}", PROVIDER_CLAUDE_EXPORT, conv.id);
    let mut turns: Vec<Turn> = Vec::new();

    for (idx, msg) in conv.messages.iter().enumerate() {
        let msg_obj = match msg.as_object() {
            Some(o) => o,
            None => continue,
        };

        let msg_id = msg_obj
            .get("uuid")
            .and_then(|v| v.as_str())
            .map(|s| format!("{}:{}:{}", PROVIDER_CLAUDE_EXPORT, conv.id, s))
            .unwrap_or_else(|| format!("{}:{}:{}", PROVIDER_CLAUDE_EXPORT, conv.id, idx));

        let timestamp = msg_obj
            .get("created_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let model = msg_obj
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Some Claude.ai exports include per-message token usage.
        let (input_tokens, output_tokens) = extract_anthropic_usage(msg);

        turns.push(Turn {
            session_id: session_id.clone(),
            provider: PROVIDER_CLAUDE_EXPORT.to_string(),
            timestamp,
            model,
            input_tokens,
            output_tokens,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
            reasoning_output_tokens: 0,
            estimated_cost_nanos: 0,
            tool_name: None,
            cwd: String::new(),
            message_id: msg_id,
            service_tier: None,
            inference_geo: None,
            is_subagent: false,
            agent_id: None,
            source_path: String::new(),
            version: None,
            pricing_version: String::new(),
            pricing_model: String::new(),
            billing_mode: "import".to_string(),
            cost_confidence: "none".to_string(),
            category: String::new(),
            all_tools: Vec::new(),
            tool_use_ids: Vec::new(),
            tool_inputs: Vec::new(),
            credits: None,
        });
    }

    let count = turns.len();
    if count == 0 {
        return Ok(0);
    }

    let sessions = build_sessions(
        &session_id,
        PROVIDER_CLAUDE_EXPORT,
        conv.title.as_deref(),
        conv.created_at.as_deref(),
        conv.updated_at.as_deref(),
        &turns,
    );

    upsert_sessions(conn, &sessions)?;
    insert_turns(conn, &turns)?;

    // Write title into sessions table.
    if let Some(title) = &conv.title {
        conn.execute(
            "UPDATE sessions SET title = ?1 WHERE session_id = ?2 AND title IS NULL",
            rusqlite::params![title, session_id],
        )?;
    }

    Ok(count)
}

/// Ingest a single OpenAI `Conversation` into the scanner DB.
/// Token counts are always 0 — the public export format omits them.
pub fn ingest_openai(conn: &Connection, conv: &super::openai::Conversation) -> Result<usize> {
    let key = super::openai::conversation_key(conv).unwrap_or_else(|| "unknown".to_string());
    let session_id = format!("{}:{}", PROVIDER_CHATGPT_EXPORT, key);

    // Walk the mapping in topological order (follow children from nodes without parents).
    let mut turns: Vec<Turn> = Vec::new();
    let mut stack: Vec<&str> = conv
        .mapping
        .values()
        .filter(|n| n.parent.is_none())
        .map(|n| n.id.as_str())
        .collect();

    let mut visited = std::collections::HashSet::new();
    while let Some(node_id) = stack.pop() {
        if !visited.insert(node_id) {
            continue;
        }
        let node = match conv.mapping.get(node_id) {
            Some(n) => n,
            None => continue,
        };
        for child in &node.children {
            stack.push(child.as_str());
        }

        let msg = match &node.message {
            Some(m) => m,
            None => continue,
        };

        let timestamp = msg.create_time.map(epoch_to_rfc3339).unwrap_or_default();

        let msg_id = format!("{}:{}:{}", PROVIDER_CHATGPT_EXPORT, key, msg.id);

        turns.push(Turn {
            session_id: session_id.clone(),
            provider: PROVIDER_CHATGPT_EXPORT.to_string(),
            timestamp,
            model: String::new(),
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
            reasoning_output_tokens: 0,
            estimated_cost_nanos: 0,
            tool_name: None,
            cwd: String::new(),
            message_id: msg_id,
            service_tier: None,
            inference_geo: None,
            is_subagent: false,
            agent_id: None,
            source_path: String::new(),
            version: None,
            pricing_version: String::new(),
            pricing_model: String::new(),
            billing_mode: "import".to_string(),
            cost_confidence: "none".to_string(),
            category: String::new(),
            all_tools: Vec::new(),
            tool_use_ids: Vec::new(),
            tool_inputs: Vec::new(),
            credits: None,
        });
    }

    let count = turns.len();
    if count == 0 {
        return Ok(0);
    }

    let first_ts = conv.create_time.map(epoch_to_rfc3339).unwrap_or_default();
    let last_ts = conv.update_time.map(epoch_to_rfc3339).unwrap_or_default();
    let sessions = build_sessions(
        &session_id,
        PROVIDER_CHATGPT_EXPORT,
        conv.title.as_deref(),
        Some(first_ts.as_str()).filter(|s| !s.is_empty()),
        Some(last_ts.as_str()).filter(|s| !s.is_empty()),
        &turns,
    );

    upsert_sessions(conn, &sessions)?;
    insert_turns(conn, &turns)?;

    if let Some(title) = &conv.title {
        conn.execute(
            "UPDATE sessions SET title = ?1 WHERE session_id = ?2 AND title IS NULL",
            rusqlite::params![title, session_id],
        )?;
    }

    Ok(count)
}

fn build_sessions(
    session_id: &str,
    provider: &str,
    title: Option<&str>,
    first_ts: Option<&str>,
    last_ts: Option<&str>,
    turns: &[Turn],
) -> Vec<Session> {
    let mut metas = crate::scanner::parser::session_metas_from_turns_pub(provider, turns);
    // Patch timestamps from the conversation-level metadata when available.
    for m in &mut metas {
        if let Some(ts) = first_ts.filter(|s| !s.is_empty()) {
            m.first_timestamp = ts.to_string();
        }
        if let Some(ts) = last_ts.filter(|s| !s.is_empty()) {
            m.last_timestamp = ts.to_string();
        }
    }
    let mut sessions = aggregate_sessions(&metas, turns);
    if let (Some(title), Some(s)) = (title, sessions.first_mut())
        && s.title.is_none()
    {
        s.title = Some(title.to_string());
    }
    if let Some(s) = sessions.first_mut()
        && s.session_id.is_empty()
    {
        s.session_id = session_id.to_string();
    }
    sessions
}

fn extract_anthropic_usage(msg: &serde_json::Value) -> (i64, i64) {
    let usage = match msg.get("usage") {
        Some(u) => u,
        None => return (0, 0),
    };
    let input = usage
        .get("input_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let output = usage
        .get("output_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    (input, output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::db::{init_db, open_db};
    use tempfile::TempDir;

    fn make_db(dir: &TempDir) -> Connection {
        let path = dir.path().join("test.db");
        let conn = open_db(&path).unwrap();
        init_db(&conn).unwrap();
        conn
    }

    #[test]
    fn ingest_anthropic_inserts_turns_and_session() {
        let dir = TempDir::new().unwrap();
        let conn = make_db(&dir);

        let conv = crate::archive::imports::anthropic::NormalizedConversation {
            id: "conv-1".to_string(),
            title: Some("Test chat".to_string()),
            created_at: Some("2026-01-01T00:00:00Z".to_string()),
            updated_at: Some("2026-01-01T01:00:00Z".to_string()),
            messages: vec![
                serde_json::json!({ "uuid": "m1", "sender": "human", "text": "hi", "created_at": "2026-01-01T00:00:00Z" }),
                serde_json::json!({ "uuid": "m2", "sender": "assistant", "text": "hello", "created_at": "2026-01-01T00:01:00Z" }),
            ],
            extras: serde_json::Value::Object(serde_json::Map::new()),
        };

        let count = ingest_anthropic(&conn, &conv).unwrap();
        assert_eq!(count, 2);

        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM turns WHERE provider = 'claude_export'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 2);

        let title: Option<String> = conn
            .query_row(
                "SELECT title FROM sessions WHERE session_id = 'claude_export:conv-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(title.as_deref(), Some("Test chat"));
    }

    #[test]
    fn ingest_openai_inserts_turns_and_session() {
        use std::collections::HashMap;
        let dir = TempDir::new().unwrap();
        let conn = make_db(&dir);

        let mut mapping = HashMap::new();
        mapping.insert(
            "n1".to_string(),
            crate::archive::imports::openai::Node {
                id: "n1".to_string(),
                message: Some(crate::archive::imports::openai::Message {
                    id: "msg-1".to_string(),
                    author: None,
                    create_time: Some(1_700_000_000.0),
                    update_time: None,
                    content: None,
                    status: None,
                    extras: HashMap::new(),
                }),
                parent: None,
                children: vec!["n2".to_string()],
            },
        );
        mapping.insert(
            "n2".to_string(),
            crate::archive::imports::openai::Node {
                id: "n2".to_string(),
                message: Some(crate::archive::imports::openai::Message {
                    id: "msg-2".to_string(),
                    author: None,
                    create_time: Some(1_700_000_060.0),
                    update_time: None,
                    content: None,
                    status: None,
                    extras: HashMap::new(),
                }),
                parent: Some("n1".to_string()),
                children: vec![],
            },
        );

        let conv = crate::archive::imports::openai::Conversation {
            id: Some("chatgpt-1".to_string()),
            conversation_id: None,
            title: Some("My GPT chat".to_string()),
            create_time: Some(1_700_000_000.0),
            update_time: Some(1_700_000_060.0),
            current_node: None,
            is_archived: None,
            mapping,
            extras: HashMap::new(),
        };

        let count = ingest_openai(&conn, &conv).unwrap();
        assert_eq!(count, 2);

        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM turns WHERE provider = 'chatgpt_export'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 2);
    }

    #[test]
    fn ingest_anthropic_idempotent() {
        let dir = TempDir::new().unwrap();
        let conn = make_db(&dir);

        let conv = crate::archive::imports::anthropic::NormalizedConversation {
            id: "conv-idem".to_string(),
            title: None,
            created_at: None,
            updated_at: None,
            messages: vec![serde_json::json!({ "uuid": "m1" })],
            extras: serde_json::Value::Object(serde_json::Map::new()),
        };

        ingest_anthropic(&conn, &conv).unwrap();
        ingest_anthropic(&conn, &conv).unwrap(); // second call must not duplicate

        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM turns WHERE session_id = 'claude_export:conv-idem'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 1, "idempotent: second ingest must not duplicate rows");
    }
}

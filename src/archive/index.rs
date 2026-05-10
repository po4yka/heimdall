//! Cross-tier SQLite index for web-conversation enrichments.
//!
//! Maintains `<archive_root>/web/index.sqlite` with normalized tables for
//! artifacts, citations, and browsing steps so they can be joined against
//! the main scanner DB by (vendor, conversation_id).

use std::path::Path;

use anyhow::Result;
use rusqlite::{params, Connection};

use crate::archive::web::WebConversation;

fn open_index(archive_root: &Path) -> Result<Connection> {
    let dir = archive_root.join("web");
    std::fs::create_dir_all(&dir)?;
    let conn = Connection::open(dir.join("index.sqlite"))?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout=5000;",
    )?;
    Ok(conn)
}

fn ensure_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS web_conversations (
            vendor              TEXT NOT NULL,
            conversation_id     TEXT NOT NULL,
            captured_at         TEXT,
            schema_fingerprint  TEXT,
            PRIMARY KEY (vendor, conversation_id)
        );
        CREATE TABLE IF NOT EXISTS web_artifacts (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            vendor              TEXT NOT NULL,
            conversation_id     TEXT NOT NULL,
            message_id          TEXT,
            identifier          TEXT,
            artifact_type       TEXT,
            language            TEXT,
            title               TEXT,
            byte_range_start    INTEGER,
            byte_range_end      INTEGER
        );
        CREATE INDEX IF NOT EXISTS idx_artifacts_conv
            ON web_artifacts(vendor, conversation_id);
        CREATE TABLE IF NOT EXISTS web_citations (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            vendor              TEXT NOT NULL,
            conversation_id     TEXT NOT NULL,
            message_id          TEXT,
            marker              TEXT,
            citation_index      TEXT,
            anchor_text         TEXT,
            url                 TEXT,
            title               TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_citations_conv
            ON web_citations(vendor, conversation_id);
        CREATE TABLE IF NOT EXISTS web_browsing_steps (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            vendor              TEXT NOT NULL,
            conversation_id     TEXT NOT NULL,
            node_id             TEXT,
            query               TEXT,
            result_summary      TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_browsing_conv
            ON web_browsing_steps(vendor, conversation_id);",
    )?;
    Ok(())
}

/// Upsert a web conversation and its extracted enrichments into the cross-tier index.
///
/// Any existing enrichment rows for this (vendor, conversation_id) pair are replaced
/// atomically so the index always reflects the latest extraction.
pub fn index_web_conversation(archive_root: &Path, conv: &WebConversation) -> Result<()> {
    let mut conn = open_index(archive_root)?;
    ensure_schema(&conn)?;

    let vendor = &conv.vendor;
    let conv_id = &conv.conversation_id;
    let extracted = conv.payload.get("heimdall_extracted");

    let tx = conn.transaction()?;

    tx.execute(
        "INSERT OR REPLACE INTO web_conversations
             (vendor, conversation_id, captured_at, schema_fingerprint)
         VALUES (?1, ?2, ?3, ?4)",
        params![vendor, conv_id, conv.captured_at, conv.schema_fingerprint],
    )?;

    tx.execute(
        "DELETE FROM web_artifacts WHERE vendor = ?1 AND conversation_id = ?2",
        params![vendor, conv_id],
    )?;
    tx.execute(
        "DELETE FROM web_citations WHERE vendor = ?1 AND conversation_id = ?2",
        params![vendor, conv_id],
    )?;
    tx.execute(
        "DELETE FROM web_browsing_steps WHERE vendor = ?1 AND conversation_id = ?2",
        params![vendor, conv_id],
    )?;

    if let Some(ex) = extracted {
        if let Some(arts) = ex.get("artifacts").and_then(|a| a.as_array()) {
            for art in arts {
                let msg_id = art.get("message_id").and_then(|v| v.as_str()).unwrap_or("");
                let identifier = art.get("identifier").and_then(|v| v.as_str()).unwrap_or("");
                let art_type = art.get("type").and_then(|v| v.as_str()).unwrap_or("");
                let language = art.get("language").and_then(|v| v.as_str()).unwrap_or("");
                let title = art.get("title").and_then(|v| v.as_str()).unwrap_or("");
                let (rs, re) = art
                    .get("byte_range")
                    .and_then(|r| r.as_array())
                    .and_then(|a| Some((a.first()?.as_i64()?, a.get(1)?.as_i64()?)))
                    .unwrap_or((0, 0));
                tx.execute(
                    "INSERT INTO web_artifacts
                         (vendor, conversation_id, message_id, identifier,
                          artifact_type, language, title, byte_range_start, byte_range_end)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                    params![vendor, conv_id, msg_id, identifier, art_type, language, title, rs, re],
                )?;
            }
        }

        if let Some(cits) = ex.get("citations").and_then(|c| c.as_array()) {
            for cit in cits {
                let msg_id = cit.get("message_id").and_then(|v| v.as_str()).unwrap_or("");
                let marker = cit.get("marker").and_then(|v| v.as_str()).unwrap_or("");
                let index = cit.get("index").and_then(|v| v.as_str()).unwrap_or("");
                let anchor = cit.get("anchor_text").and_then(|v| v.as_str()).unwrap_or("");
                let url = cit.get("url").and_then(|v| v.as_str()).unwrap_or("");
                let title = cit.get("title").and_then(|v| v.as_str()).unwrap_or("");
                tx.execute(
                    "INSERT INTO web_citations
                         (vendor, conversation_id, message_id, marker,
                          citation_index, anchor_text, url, title)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                    params![vendor, conv_id, msg_id, marker, index, anchor, url, title],
                )?;
            }
        }

        if let Some(steps) = ex.get("browsing_steps").and_then(|s| s.as_array()) {
            for step in steps {
                let node_id = step.get("node_id").and_then(|v| v.as_str()).unwrap_or("");
                let query = step.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let summary: String = step
                    .get("result")
                    .and_then(|v| v.as_str())
                    .map(|s| s.chars().take(200).collect())
                    .unwrap_or_default();
                tx.execute(
                    "INSERT INTO web_browsing_steps
                         (vendor, conversation_id, node_id, query, result_summary)
                     VALUES (?1,?2,?3,?4,?5)",
                    params![vendor, conv_id, node_id, query, summary],
                )?;
            }
        }
    }

    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    fn make_conv(vendor: &str, id: &str, payload: serde_json::Value) -> WebConversation {
        WebConversation {
            vendor: vendor.to_string(),
            conversation_id: id.to_string(),
            captured_at: "2025-01-01T00:00:00Z".to_string(),
            schema_fingerprint: "abc".to_string(),
            payload,
        }
    }

    #[test]
    fn indexes_artifacts_with_language() {
        let tmp = TempDir::new().unwrap();
        let c = make_conv("claude.ai", "conv-1", json!({
            "heimdall_extracted": {
                "artifacts": [{
                    "message_id": "m1",
                    "identifier": "foo",
                    "type": "application/vnd.ant.code",
                    "language": "rust",
                    "title": "Hello",
                    "body": "fn main() {}",
                    "byte_range": [0, 50]
                }],
                "citations": [],
                "browsing_steps": []
            }
        }));
        index_web_conversation(tmp.path(), &c).unwrap();

        let conn = Connection::open(tmp.path().join("web/index.sqlite")).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM web_artifacts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
        let lang: String = conn
            .query_row(
                "SELECT language FROM web_artifacts WHERE identifier='foo'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(lang, "rust");
    }

    #[test]
    fn replace_on_reindex() {
        let tmp = TempDir::new().unwrap();
        let art = |id: &str| json!({
            "message_id": "m1", "identifier": id, "type": "text",
            "language": "", "title": "T", "body": "", "byte_range": [0, 1]
        });
        let c1 = make_conv("claude.ai", "conv-1", json!({
            "heimdall_extracted": { "artifacts": [art("a")], "citations": [], "browsing_steps": [] }
        }));
        let c2 = make_conv("claude.ai", "conv-1", json!({
            "heimdall_extracted": { "artifacts": [art("b"), art("c")], "citations": [], "browsing_steps": [] }
        }));
        index_web_conversation(tmp.path(), &c1).unwrap();
        index_web_conversation(tmp.path(), &c2).unwrap();

        let conn = Connection::open(tmp.path().join("web/index.sqlite")).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM web_artifacts WHERE conversation_id='conv-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn no_heimdall_extracted_is_ok() {
        let tmp = TempDir::new().unwrap();
        let c = make_conv("chatgpt.com", "c2", json!({ "title": "bare" }));
        index_web_conversation(tmp.path(), &c).unwrap();

        let conn = Connection::open(tmp.path().join("web/index.sqlite")).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM web_conversations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
}

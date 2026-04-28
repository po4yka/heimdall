//! Anthropic claude.ai export parser.
//!
//! The export schema is not publicly documented as of 2026-04. We parse
//! defensively: any JSON file in the ZIP becomes a candidate; we extract
//! a normalized `(id, title, timestamps, messages[])` view via a best-
//! effort field-name probe, and preserve everything else under `extras`.
//! `metadata.json` records the schema fingerprint so a future Heimdall
//! version can offer to re-parse old imports under a tighter parser.

use std::collections::BTreeSet;
use std::io::Read;

use anyhow::{Context, Result};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct NormalizedConversation {
    pub id: String,
    pub title: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub messages: Vec<Value>,
    pub extras: Value, // entire original object preserved verbatim
}

/// Best-effort extraction of a normalized conversation from a JSON value.
pub fn normalize(value: Value) -> Option<NormalizedConversation> {
    let obj = value.as_object()?.clone();
    let id = pick_str(&obj, &["uuid", "conversation_id", "id"])?;
    let title = pick_str(&obj, &["name", "title", "summary"]);
    let created_at = pick_str(&obj, &["created_at", "createdAt", "create_time"]);
    let updated_at = pick_str(&obj, &["updated_at", "updatedAt", "update_time"]);
    let messages = pick_array(&obj, &["chat_messages", "messages", "turns"]).unwrap_or_default();
    Some(NormalizedConversation {
        id,
        title,
        created_at,
        updated_at,
        messages,
        extras: Value::Object(obj),
    })
}

fn pick_str(obj: &Map<String, Value>, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(v) = obj.get(*k)
            && let Some(s) = v.as_str()
        {
            return Some(s.to_string());
        }
    }
    None
}

fn pick_array(obj: &Map<String, Value>, keys: &[&str]) -> Option<Vec<Value>> {
    for k in keys {
        if let Some(v) = obj.get(*k)
            && let Some(arr) = v.as_array()
        {
            return Some(arr.clone());
        }
    }
    None
}

/// Compute a schema fingerprint: SHA-256 of the sorted set of top-level
/// JSON keys observed across all conversations.
pub fn schema_fingerprint(values: &[Value]) -> String {
    let mut keys: BTreeSet<String> = BTreeSet::new();
    for v in values {
        if let Some(obj) = v.as_object() {
            for k in obj.keys() {
                keys.insert(k.clone());
            }
        }
    }
    let mut hasher = Sha256::new();
    for k in &keys {
        hasher.update(k.as_bytes());
        hasher.update(b"\n");
    }
    let bytes = hasher.finalize();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Read every .json entry from a ZIP and yield `(filename, parsed Value)`.
pub fn read_json_entries_from_zip<R: Read + std::io::Seek>(
    zip: &mut zip::ZipArchive<R>,
) -> Result<Vec<(String, Value)>> {
    let mut out = Vec::new();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        if !entry.name().ends_with(".json") {
            continue;
        }
        let name = entry.name().to_string();
        let mut buf = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut buf)?;
        let value: Value =
            serde_json::from_slice(&buf).with_context(|| format!("parsing {} as JSON", name))?;
        out.push((name, value));
    }
    Ok(out)
}

/// Walk a parsed JSON value and extract every top-level conversation it
/// contains. Handles three common shapes:
///   - top-level array of conversations
///   - top-level object whose `conversations` key is an array
///   - top-level single-conversation object
pub fn collect_conversations(value: &Value) -> Vec<Value> {
    if let Some(arr) = value.as_array() {
        return arr.clone();
    }
    if let Some(obj) = value.as_object() {
        for k in &["conversations", "chat_conversations", "data"] {
            if let Some(v) = obj.get(*k)
                && let Some(arr) = v.as_array()
            {
                return arr.clone();
            }
        }
        // Treat the whole object as a single conversation if it has an id-ish field.
        if obj
            .keys()
            .any(|k| ["uuid", "conversation_id", "id"].contains(&k.as_str()))
        {
            return vec![value.clone()];
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_picks_uuid_and_messages() {
        let v = serde_json::json!({
            "uuid": "abc",
            "name": "My chat",
            "created_at": "2026-04-01T00:00:00Z",
            "chat_messages": [{"text": "hi"}],
            "extra_thing": 42
        });
        let c = normalize(v).unwrap();
        assert_eq!(c.id, "abc");
        assert_eq!(c.title.as_deref(), Some("My chat"));
        assert_eq!(c.messages.len(), 1);
        assert!(c.extras.get("extra_thing").is_some());
    }

    #[test]
    fn normalize_returns_none_without_id() {
        let v = serde_json::json!({"name": "untitled"});
        assert!(normalize(v).is_none());
    }

    #[test]
    fn collect_conversations_handles_array_and_object_shapes() {
        let arr = serde_json::json!([{"uuid":"a"},{"uuid":"b"}]);
        assert_eq!(collect_conversations(&arr).len(), 2);

        let wrapped = serde_json::json!({"conversations":[{"uuid":"a"}]});
        assert_eq!(collect_conversations(&wrapped).len(), 1);

        let single = serde_json::json!({"uuid":"x","name":"n"});
        assert_eq!(collect_conversations(&single).len(), 1);
    }

    #[test]
    fn schema_fingerprint_is_stable_across_field_order() {
        let v1 = vec![serde_json::json!({"a":1, "b":2})];
        let v2 = vec![serde_json::json!({"b":2, "a":1})];
        assert_eq!(schema_fingerprint(&v1), schema_fingerprint(&v2));
    }
}

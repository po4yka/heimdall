//! Extract `<antartifact>` blocks from Claude `chat_messages[].text`.
//!
//! Called at write-time by the POST handler and at import-time by the ZIP
//! importer. Results are written into `payload["heimdall_extracted"]["artifacts"]`
//! so the struct schema (`WebConversation`) stays stable.

use std::sync::OnceLock;

use anyhow::Result;
use regex::Regex;
use serde_json::{json, Value};

fn artifact_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?s)<antartifact([^>]*)>(.*?)</antartifact>").unwrap()
    })
}

fn attr_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"([\w-]+)="([^"]*)""#).unwrap())
}

/// Populate `payload["heimdall_extracted"]["artifacts"]` from `chat_messages[].text`.
///
/// Idempotent: returns immediately if `artifacts` is already present.
/// Errors are surfaced to callers; the POST handler treats them as best-effort.
pub fn extract(payload: &mut Value) -> Result<()> {
    if payload
        .get("heimdall_extracted")
        .and_then(|e| e.get("artifacts"))
        .and_then(|a| a.as_array())
        .is_some()
    {
        return Ok(());
    }

    let mut artifacts: Vec<Value> = Vec::new();

    if let Some(messages) = payload
        .get("chat_messages")
        .and_then(|m| m.as_array())
        .cloned()
    {
        for msg in &messages {
            let msg_id = msg
                .get("uuid")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if let Some(text) = msg.get("text").and_then(|t| t.as_str()) {
                for cap in artifact_re().captures_iter(text) {
                    let attrs_str = cap.get(1).map_or("", |m| m.as_str());
                    let body = cap.get(2).map_or("", |m| m.as_str());
                    let full = cap.get(0).unwrap();

                    let mut identifier = String::new();
                    let mut artifact_type = String::new();
                    let mut title = String::new();
                    let mut language = String::new();

                    for attr in attr_re().captures_iter(attrs_str) {
                        let key = attr.get(1).map_or("", |m| m.as_str());
                        let val = attr.get(2).map_or("", |m| m.as_str());
                        match key {
                            "identifier" => identifier = val.to_string(),
                            "type" => artifact_type = val.to_string(),
                            "title" => title = val.to_string(),
                            "language" => language = val.to_string(),
                            _ => {}
                        }
                    }

                    artifacts.push(json!({
                        "message_id": msg_id,
                        "identifier": identifier,
                        "type": artifact_type,
                        "language": language,
                        "title": title,
                        "body": body,
                        "byte_range": [full.start(), full.end()],
                    }));
                }
            }
        }
    }

    let extracted = payload
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("payload is not a JSON object"))?
        .entry("heimdall_extracted")
        .or_insert_with(|| json!({}));

    extracted["artifacts"] = json!(artifacts);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_payload(messages: serde_json::Value) -> Value {
        json!({ "chat_messages": messages })
    }

    #[test]
    fn extracts_two_artifacts() {
        let mut payload = make_payload(json!([{
            "uuid": "msg-1",
            "text": "Here is an artifact:\n<antartifact identifier=\"foo\" type=\"text/html\" title=\"My Page\"><h1>Hello</h1></antartifact>\nAnd another:\n<antartifact identifier=\"bar\" type=\"application/vnd.ant.code\" title=\"Code\">fn main() {}</antartifact>"
        }]));
        extract(&mut payload).unwrap();
        let arts = payload["heimdall_extracted"]["artifacts"].as_array().unwrap();
        assert_eq!(arts.len(), 2);
        assert_eq!(arts[0]["identifier"], "foo");
        assert_eq!(arts[0]["type"], "text/html");
        assert_eq!(arts[0]["title"], "My Page");
        assert_eq!(arts[0]["message_id"], "msg-1");
        assert!(arts[0]["body"].as_str().unwrap().contains("Hello"));
        assert_eq!(arts[1]["identifier"], "bar");
    }

    #[test]
    fn no_artifacts_writes_empty_array() {
        let mut payload = make_payload(json!([{
            "uuid": "msg-1",
            "text": "No artifacts here, just text."
        }]));
        extract(&mut payload).unwrap();
        let arts = payload["heimdall_extracted"]["artifacts"].as_array().unwrap();
        assert!(arts.is_empty());
    }

    #[test]
    fn idempotent_on_second_call() {
        let mut payload = make_payload(json!([{
            "uuid": "msg-1",
            "text": "<antartifact identifier=\"x\" type=\"text\" title=\"T\">body</antartifact>"
        }]));
        extract(&mut payload).unwrap();
        // Manually tamper to verify the second call does not re-extract.
        payload["heimdall_extracted"]["marker"] = json!(true);
        extract(&mut payload).unwrap();
        assert_eq!(payload["heimdall_extracted"]["marker"], true);
    }

    #[test]
    fn multiline_artifact_body() {
        let mut payload = make_payload(json!([{
            "uuid": "msg-1",
            "text": "<antartifact identifier=\"doc\" type=\"text/markdown\" title=\"Doc\">\n# Heading\n\nParagraph.\n</antartifact>"
        }]));
        extract(&mut payload).unwrap();
        let arts = payload["heimdall_extracted"]["artifacts"].as_array().unwrap();
        assert_eq!(arts.len(), 1);
        assert!(arts[0]["body"].as_str().unwrap().contains("Heading"));
    }
}

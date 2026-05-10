//! Extract browsing steps and citation markers from ChatGPT conversation payloads.
//!
//! Results land in `payload["heimdall_extracted"]`:
//!   - `browsing_steps`: from `tether_browsing_display` mapping nodes.
//!   - `citations`: opaque markers parsed from assistant message text.
//!     If `citations` is already populated (e.g. set by the browser extension
//!     which has access to the sidebar DOM), it is left untouched ("extension wins").

use std::sync::OnceLock;

use anyhow::Result;
use regex::Regex;
use serde_json::{json, Value};

fn citation_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Matches: 【57†L18-L22】 or 【3†L5】
    RE.get_or_init(|| Regex::new(r"【(\d+)†(L\d+(?:-L\d+)?)】").unwrap())
}

/// Populate `payload["heimdall_extracted"]` with `browsing_steps` and `citations`.
///
/// - Idempotent: skips if `browsing_steps` is already present.
/// - Extension-wins: skips `citations` if already populated (sidebar scrape took precedence).
pub fn extract(payload: &mut Value) -> Result<()> {
    let already_has_steps = payload
        .get("heimdall_extracted")
        .and_then(|e| e.get("browsing_steps"))
        .and_then(|s| s.as_array())
        .is_some();
    if already_has_steps {
        return Ok(());
    }

    let citations_already_set = payload
        .get("heimdall_extracted")
        .and_then(|e| e.get("citations"))
        .and_then(|c| c.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false);

    let mapping = match payload.get("mapping").and_then(|m| m.as_object()).cloned() {
        Some(m) => m,
        None => {
            // No mapping tree; write empty arrays and return.
            let ex = payload
                .as_object_mut()
                .ok_or_else(|| anyhow::anyhow!("payload is not a JSON object"))?
                .entry("heimdall_extracted")
                .or_insert_with(|| json!({}));
            ex["browsing_steps"] = json!([]);
            if !citations_already_set {
                ex["citations"] = json!([]);
            }
            return Ok(());
        }
    };

    let mut browsing_steps: Vec<Value> = Vec::new();
    let mut citations: Vec<Value> = Vec::new();

    for (node_id, node) in &mapping {
        let message = match node.get("message") {
            Some(m) if !m.is_null() => m,
            _ => continue,
        };

        let content = match message.get("content") {
            Some(c) => c,
            None => continue,
        };

        let content_type = content
            .get("content_type")
            .and_then(|ct| ct.as_str())
            .unwrap_or("");

        match content_type {
            "tether_browsing_display" => {
                // Extract the browsing query and result list from the content.
                let result_value = content.get("result").cloned().unwrap_or(Value::Null);
                let query = content
                    .get("tether_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // The browsing results live under different keys depending on the API version.
                let results = content
                    .get("results")
                    .cloned()
                    .or_else(|| content.get("items").cloned())
                    .unwrap_or_else(|| json!([]));

                browsing_steps.push(json!({
                    "node_id": node_id,
                    "query": query,
                    "result": result_value,
                    "results": results,
                }));
            }
            _ => {
                // Look for citation markers in assistant text parts.
                let role = message
                    .get("author")
                    .and_then(|a| a.get("role"))
                    .and_then(|r| r.as_str())
                    .unwrap_or("");

                if role != "assistant" {
                    continue;
                }

                let msg_id = message.get("id").and_then(|v| v.as_str()).unwrap_or("");

                if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                    for part in parts {
                        if let Some(text) = part.as_str() {
                            for cap in citation_re().captures_iter(text) {
                                let index = cap.get(1).map_or("", |m| m.as_str());
                                let anchor = cap.get(2).map_or("", |m| m.as_str());
                                citations.push(json!({
                                    "marker": cap.get(0).map_or("", |m| m.as_str()),
                                    "index": index,
                                    "anchor_text": anchor,
                                    "message_id": msg_id,
                                }));
                            }
                        }
                    }
                }
            }
        }
    }

    let ex = payload
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("payload is not a JSON object"))?
        .entry("heimdall_extracted")
        .or_insert_with(|| json!({}));

    ex["browsing_steps"] = json!(browsing_steps);
    if !citations_already_set {
        ex["citations"] = json!(citations);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn node(id: &str, role: &str, content_type: &str, parts: Value) -> (String, Value) {
        (
            id.to_string(),
            json!({
                "id": id,
                "parent": null,
                "children": [],
                "message": {
                    "id": id,
                    "author": { "role": role },
                    "content": {
                        "content_type": content_type,
                        "parts": parts,
                    }
                }
            }),
        )
    }

    fn browsing_node(id: &str, tether_id: &str, results: Value) -> (String, Value) {
        (
            id.to_string(),
            json!({
                "id": id,
                "parent": null,
                "children": [],
                "message": {
                    "id": id,
                    "author": { "role": "tool" },
                    "content": {
                        "content_type": "tether_browsing_display",
                        "tether_id": tether_id,
                        "result": "search result text",
                        "results": results,
                    }
                }
            }),
        )
    }

    #[test]
    fn extracts_browsing_step_and_citations() {
        let (bid, bnode) = browsing_node("n1", "query-abc", json!([{"title":"Page","url":"https://example.com"}]));
        let (aid, anode) = node("n2", "assistant", "text", json!(["Hello 【1†L1-L4】 and 【2†L7】"]));

        let mapping: serde_json::Map<String, Value> = [(bid, bnode), (aid, anode)].into_iter().collect();
        let mut payload = json!({ "mapping": mapping });

        extract(&mut payload).unwrap();

        let steps = payload["heimdall_extracted"]["browsing_steps"].as_array().unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0]["node_id"], "n1");

        let cits = payload["heimdall_extracted"]["citations"].as_array().unwrap();
        assert_eq!(cits.len(), 2);
        assert_eq!(cits[0]["index"], "1");
        assert_eq!(cits[0]["anchor_text"], "L1-L4");
        assert_eq!(cits[1]["index"], "2");
    }

    #[test]
    fn extension_wins_leaves_citations_untouched() {
        let (aid, anode) = node("n1", "assistant", "text", json!(["Text 【5†L3】"]));
        let mapping: serde_json::Map<String, Value> = [(aid, anode)].into_iter().collect();
        let mut payload = json!({
            "mapping": mapping,
            "heimdall_extracted": {
                "citations": [{"marker": "【5†L3】", "index": "5", "url": "https://example.com"}]
            }
        });

        extract(&mut payload).unwrap();

        // Extractor must not overwrite the extension-supplied citation with url.
        let cits = payload["heimdall_extracted"]["citations"].as_array().unwrap();
        assert_eq!(cits.len(), 1);
        assert_eq!(cits[0]["url"], "https://example.com");
    }

    #[test]
    fn idempotent_on_second_call() {
        let (bid, bnode) = browsing_node("n1", "q", json!([]));
        let mapping: serde_json::Map<String, Value> = [(bid, bnode)].into_iter().collect();
        let mut payload = json!({ "mapping": mapping });

        extract(&mut payload).unwrap();
        payload["heimdall_extracted"]["marker"] = json!(42);
        extract(&mut payload).unwrap();

        assert_eq!(payload["heimdall_extracted"]["marker"], 42);
    }

    #[test]
    fn no_mapping_writes_empty_arrays() {
        let mut payload = json!({ "title": "no mapping" });
        extract(&mut payload).unwrap();
        assert!(payload["heimdall_extracted"]["browsing_steps"].as_array().unwrap().is_empty());
        assert!(payload["heimdall_extracted"]["citations"].as_array().unwrap().is_empty());
    }
}

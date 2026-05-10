//! Vendor-specific payload extractors.
//!
//! Each extractor receives a mutable reference to `payload: &mut serde_json::Value`
//! and writes enrichment data into `payload["heimdall_extracted"]`. The parent
//! `WebConversation` struct is unchanged; enrichments ride inside the opaque payload.

pub mod anthropic;
pub mod openai;

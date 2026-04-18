// Library root — exposes internal modules for use by both binary targets
// (claude-usage-tracker and heimdall-hook).
//
// The main binary (src/main.rs) re-declares these modules with `mod` so they
// are compiled as part of the binary's crate root. The hook binary
// (src/hook/main.rs) uses `use claude_usage_tracker::hook` to reach the
// shared code compiled here.

pub mod config;
pub mod currency;
pub mod export;
pub mod hook;
pub mod litellm;
pub mod menubar;
pub mod models;
pub mod oauth;
pub mod openai;
pub mod optimizer;
pub mod pricing;
pub mod scanner;
pub mod scheduler;
pub mod server;
pub mod tz;
pub mod webhooks;

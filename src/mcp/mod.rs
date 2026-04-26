/// Heimdall MCP server — Phase 6.
///
/// Exposes 9 tools over stdio or HTTP (streamable-HTTP / SSE) using the
/// official `rmcp` Rust SDK.
///
/// Entry points:
///   - `run_stdio(db_path)`                   — reads from stdin, writes to stdout
///   - `run_http(host, port, db_path)` — binds an axum HTTP listener at `/mcp`
pub mod install;
#[cfg(test)]
mod tests;
mod tools;

use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;

use tools::HeimdallMcpServer;

// ── Transport selector ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum McpTransport {
    Stdio,
    Http,
}

impl std::str::FromStr for McpTransport {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "stdio" => Ok(McpTransport::Stdio),
            "http" => Ok(McpTransport::Http),
            other => Err(format!(
                "unknown transport '{}': expected stdio | http",
                other
            )),
        }
    }
}

// ── Stdio entry point ─────────────────────────────────────────────────────────

/// Run the MCP server over stdio.
///
/// Reads JSON-RPC from stdin, writes responses to stdout.
/// Blocks until the client disconnects (EOF on stdin).
/// Never panics on malformed JSON-RPC — rmcp handles all protocol errors.
pub async fn run_stdio(db_path: PathBuf) -> Result<()> {
    use rmcp::ServiceExt;
    use tokio::io::{stdin, stdout};

    let cfg = crate::config::load_config_resolved();
    let statusline = cfg.resolved_statusline();
    let server = HeimdallMcpServer {
        db_path,
        default_session_length_hours: cfg.resolved_session_length(None, None),
        burn_rate_config: crate::analytics::burn_rate::BurnRateConfig::from_thresholds(
            statusline.burn_rate_normal_max,
            statusline.burn_rate_moderate_max,
        ),
    };
    let svc = server.serve((stdin(), stdout())).await?;
    // Wait for client disconnect (EOF on stdin).
    if let Err(e) = svc.waiting().await {
        tracing::warn!("MCP stdio service exited with error: {e}");
    }
    Ok(())
}

// ── HTTP entry point ──────────────────────────────────────────────────────────

/// Run the MCP server over HTTP.
///
/// Mounts a `/mcp` route on a fresh axum listener (separate from the dashboard).
/// Each POST is handled as a single-request MCP exchange; responses are SSE.
/// HTTP transport is limited to loopback binds because the endpoint has no
/// authentication layer.
pub async fn run_http(host: &str, port: u16, db_path: PathBuf) -> Result<()> {
    use axum::Router;
    use axum::routing::post;

    if !is_loopback_bind_host(host) {
        anyhow::bail!("refusing non-loopback MCP HTTP bind '{host}'; use stdio or a loopback host");
    }

    let db = Arc::new(db_path);

    let app = Router::new().route(
        "/mcp",
        post({
            let db = db.clone();
            move |req: axum::http::Request<axum::body::Body>| {
                let db = db.clone();
                async move { mcp_http_handler(req, db).await }
            }
        }),
    );

    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("MCP HTTP server at http://{}/mcp", addr);
    eprintln!("MCP HTTP server at http://{addr}/mcp");
    axum::serve(listener, app).await?;
    Ok(())
}

fn is_loopback_bind_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }

    let trimmed = host.trim().trim_matches('[').trim_matches(']');
    trimmed
        .parse::<IpAddr>()
        .is_ok_and(|addr| addr.is_loopback())
}

// ── HTTP handler ──────────────────────────────────────────────────────────────

/// Accepts a single JSON-RPC request, runs it through an in-memory rmcp
/// service, and returns the response formatted as an SSE event.
///
/// Stateless per-request model — a fresh server is created for each POST.
async fn mcp_http_handler(
    req: axum::http::Request<axum::body::Body>,
    db_path: Arc<PathBuf>,
) -> axum::response::Response {
    use axum::body::Body;
    use axum::http::{HeaderValue, StatusCode};
    use axum::response::Response;

    // --- Read and parse the body ---
    let body_bytes = match axum::body::to_bytes(req.into_body(), 4 * 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("body read error"))
                .unwrap();
        }
    };

    // Validate it's parseable JSON-RPC (we pass raw bytes through to rmcp).
    if serde_json::from_slice::<serde_json::Value>(&body_bytes).is_err() {
        let err = serde_json::json!({
            "jsonrpc": "2.0",
            "id": null,
            "error": { "code": -32700, "message": "Parse error: invalid JSON" }
        });
        return Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&err).unwrap()))
            .unwrap();
    }

    // --- Spin up an in-memory rmcp service via async_rw transport ---
    // We create a pair of in-memory pipes and feed the request into the write end.
    let (client_write, server_read) = tokio::io::duplex(64 * 1024);
    let (server_write, mut client_read) = tokio::io::duplex(64 * 1024);

    let cfg = crate::config::load_config_resolved();
    let statusline = cfg.resolved_statusline();
    let server = HeimdallMcpServer {
        db_path: (*db_path).clone(),
        default_session_length_hours: cfg.resolved_session_length(None, None),
        burn_rate_config: crate::analytics::burn_rate::BurnRateConfig::from_thresholds(
            statusline.burn_rate_normal_max,
            statusline.burn_rate_moderate_max,
        ),
    };

    use rmcp::ServiceExt;
    use tokio::io::AsyncWriteExt;

    // Write the request + newline (rmcp uses newline-delimited JSON).
    let mut writer = client_write;
    let mut msg_with_newline = body_bytes.to_vec();
    msg_with_newline.push(b'\n');
    if let Err(e) = writer.write_all(&msg_with_newline).await {
        return error_response(format!("write error: {e}"));
    }
    // Don't close writer yet — rmcp needs the stream to stay open briefly.

    let svc = match server.serve((server_read, server_write)).await {
        Ok(s) => s,
        Err(e) => return error_response(format!("server start error: {e}")),
    };

    // Signal EOF so the server knows there are no more requests, then wait for it to finish.
    drop(writer);
    if let Err(e) = svc.waiting().await {
        tracing::warn!("MCP HTTP service exited with error: {e}");
    }

    // Read the response from the server's write half.
    let mut response_bytes = Vec::new();
    use tokio::io::AsyncReadExt;
    let _ = client_read.read_to_end(&mut response_bytes).await;

    if response_bytes.is_empty() {
        return Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .unwrap();
    }

    // Extract the first line (first JSON-RPC response).
    let response_text = String::from_utf8_lossy(&response_bytes);
    let first_line = response_text.lines().next().unwrap_or("{}");

    // Return as SSE for MCP HTTP protocol compatibility.
    let sse_body = format!("event: message\ndata: {first_line}\n\n");
    Response::builder()
        .status(StatusCode::OK)
        .header(
            "Content-Type",
            HeaderValue::from_static("text/event-stream"),
        )
        .body(Body::from(sse_body))
        .unwrap()
}

fn error_response(msg: String) -> axum::response::Response {
    use axum::body::Body;
    use axum::http::StatusCode;
    let err = serde_json::json!({
        "jsonrpc": "2.0",
        "id": null,
        "error": { "code": -32000, "message": msg }
    });
    axum::response::Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&err).unwrap()))
        .unwrap()
}

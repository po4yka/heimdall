pub mod api;
pub mod credentials;
pub mod models;

use models::UsageWindowsResponse;
use tracing::debug;

/// Poll OAuth usage: load credentials, fetch from API, attach identity.
/// Returns `UsageWindowsResponse` with `available: false` if credentials missing or expired.
pub async fn poll_usage() -> UsageWindowsResponse {
    let env = std::env::vars().collect::<Vec<_>>();
    let resolved = credentials::resolve_auth(&env);
    let creds = match resolved.credentials {
        Some(c) => c,
        None => {
            debug!("No compatible Claude OAuth credentials found");
            return resolved
                .health
                .failure_reason
                .clone()
                .map(UsageWindowsResponse::with_error)
                .unwrap_or_else(UsageWindowsResponse::unavailable);
        }
    };

    let token = match credentials::get_access_token(&creds) {
        Some(t) => t.to_string(),
        None => {
            // Token expired or missing -- try to refresh if we have a refresh_token
            if creds.refresh_token.is_some() {
                debug!("OAuth token expired, attempting refresh");
                match credentials::refresh_token(&creds).await {
                    Some(new_token) => new_token,
                    None => {
                        return UsageWindowsResponse::with_error(
                            "OAuth token expired and refresh failed. Run `claude login` to refresh."
                                .into(),
                        );
                    }
                }
            } else {
                debug!("OAuth token expired or missing, no refresh token available");
                return UsageWindowsResponse::with_error(
                    "OAuth token expired. Run `claude login` to refresh.".into(),
                );
            }
        }
    };

    let identity = resolved
        .identity
        .unwrap_or_else(|| credentials::get_identity(&creds));
    let resp = api::fetch_usage(&token).await;
    api::with_identity(resp, identity)
}

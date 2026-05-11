//! Request timeout middleware and response payloads.
use crate::middleware::request_id;
use axum::{
    Json,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::time::Duration;
use tokio::time::timeout;

/// Response payload returned when a request times out.
#[derive(Debug, Serialize)]
struct TimeoutBody {
    /// Machine-readable error code.
    error: &'static str,
    /// Human-readable error message.
    message: &'static str,
    /// Optional request ID for trace correlation.
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,
}

/// Enforces a per-request timeout and returns a 408 response on expiry.
pub async fn timeout_middleware(
    request: Request,
    next: Next,
    request_timeout_secs: u64,
) -> Response {
    let request_id = request_id::request_id_from_headers(request.headers());

    match timeout(Duration::from_secs(request_timeout_secs), next.run(request)).await {
        Ok(response) => response,
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(TimeoutBody {
                error: "request_timeout",
                message: "request timed out",
                request_id,
            }),
        )
            .into_response(),
    }
}

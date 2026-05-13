use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{response::error::AppError, state::AppState};

pub async fn require_auth(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let session_token = extract_session_token(&request)
        .ok_or_else(|| AppError::unauthorized("missing or invalid credentials"))?;

    let session = state
        .auth
        .inner()
        .session_manager()
        .get_session(&session_token)
        .await
        .map_err(|_| AppError::unauthorized("invalid or expired session"))?
        .ok_or_else(|| AppError::unauthorized("invalid or expired session"))?;

    // Attach user_id to the current trace span.
    tracing::Span::current().record("user_id", tracing::field::display(&session.user_id));

    Ok(next.run(request).await)
}

fn extract_session_token(request: &Request) -> Option<String> {
    // 1. Authorization: Bearer <token>
    if let Some(token) = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_string)
    {
        return Some(token);
    }

    // 2. Cookie: better-auth.session-token=<token> or session=<token>
    request
        .headers()
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())?
        .split(';')
        .find_map(|cookie| {
            let cookie = cookie.trim();
            if let Some(v) = cookie.strip_prefix("better-auth.session-token=") {
                Some(v.to_string())
            } else if let Some(v) = cookie.strip_prefix("session=") {
                Some(v.to_string())
            } else {
                None
            }
        })
}

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::state::AppState;

pub async fn require_auth(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract cookies
    let cookies = request
        .headers()
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Find session cookie
    let session_token = cookies
        .split(';')
        .find_map(|cookie| {
            let cookie = cookie.trim();

            if cookie.starts_with("better-auth.session-token=") {
                Some(
                    cookie
                        .trim_start_matches("better-auth.session-token=")
                        .to_string(),
                )
            } else if cookie.starts_with("session=") {
                Some(cookie.trim_start_matches("session=").to_string())
            } else {
                None
            }
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate session
    state
        .auth
        .inner()
        .session_manager()
        .get_session(&session_token)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(next.run(request).await)
}

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};

use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser;

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract cookies
        let cookies = parts
            .headers
            .get(axum::http::header::COOKIE)
            .and_then(|v| v.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // Extract session token
        let session_token = cookies
            .split(';')
            .find_map(|cookie| {
                let cookie = cookie.trim();

                if cookie.starts_with("session=") {
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

        Ok(AuthenticatedUser)
    }
}

//! HTTP system handlers — convert domain types to HTTP responses.

use crate::{response::*, services::system as svc, state::AppState};
use axum::{Json, extract::State, http::StatusCode};
use macros::{instrument_handler, route};

#[route(GET, "/health")]
#[instrument_handler]
pub async fn health() -> Json<ProbeResponse> {
    Json(svc::livez().into())
}

#[route(GET, "/healthz")]
#[instrument_handler]
pub async fn healthz() -> Json<ProbeResponse> {
    Json(svc::livez().into())
}

#[route(GET, "/livez")]
#[instrument_handler]
pub async fn livez() -> Json<ProbeResponse> {
    Json(svc::livez().into())
}

#[route(GET, "/readyz")]
#[instrument_handler]
pub async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<ProbeResponse>) {
    let (status, result) = svc::readyz(&state).await;
    (status, Json(result.into()))
}

#[route(GET, "/status")]
#[instrument_handler]
pub async fn status(State(state): State<AppState>) -> (StatusCode, Json<StatusResponse>) {
    let (status, result) = svc::status(&state).await;
    (status, Json(result.into()))
}

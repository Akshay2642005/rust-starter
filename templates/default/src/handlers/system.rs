//! HTTP system handlers — convert domain types to HTTP responses.

use crate::{response::system::*, services::system as svc, state::AppState};
use axum::{Json, extract::State, http::StatusCode};
use macros::{instrument_handler, route};

#[utoipa::path(get, path = "/health", tag = "system",
    responses((status = 200, description = "Liveness probe", body = ProbeResponse)))]
#[route(GET, "/health")]
#[instrument_handler]
pub async fn health() -> Json<ProbeResponse> {
    Json(svc::livez().into())
}

#[utoipa::path(get, path = "/healthz", tag = "system",
    responses((status = 200, description = "Liveness probe", body = ProbeResponse)))]
#[route(GET, "/healthz")]
#[instrument_handler]
pub async fn healthz() -> Json<ProbeResponse> {
    Json(svc::livez().into())
}

#[utoipa::path(get, path = "/livez", tag = "system",
    responses((status = 200, description = "Liveness probe", body = ProbeResponse)))]
#[route(GET, "/livez")]
#[instrument_handler]
pub async fn livez() -> Json<ProbeResponse> {
    Json(svc::livez().into())
}

#[utoipa::path(get, path = "/readyz", tag = "system",
    responses((status = 200, description = "Readiness probe", body = ProbeResponse),
              (status = 503, description = "Not ready", body = ProbeResponse)))]
#[route(GET, "/readyz")]
#[instrument_handler]
pub async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<ProbeResponse>) {
    let (status, result) = svc::readyz(&state).await;
    (status, Json(result.into()))
}

#[utoipa::path(get, path = "/status", tag = "system",
    responses((status = 200, description = "Service status", body = StatusResponse),
              (status = 503, description = "Degraded", body = StatusResponse)))]
#[route(GET, "/status")]
#[instrument_handler]
pub async fn status(State(state): State<AppState>) -> (StatusCode, Json<StatusResponse>) {
    let (status, result) = svc::status(&state).await;
    (status, Json(result.into()))
}

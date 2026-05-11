//! System service — produces domain types, no HTTP concerns.

use crate::{domain::health::*, state::AppState};
use axum::http::StatusCode;
use chrono::Utc;

pub fn livez() -> ProbeResult {
    ProbeResult {
        status: ProbeStatus::Ok,
        timestamp: Utc::now(),
    }
}

pub async fn readyz(state: &AppState) -> (StatusCode, ProbeResult) {
    match state.db.ping().await {
        Ok(()) => (
            StatusCode::OK,
            ProbeResult {
                status: ProbeStatus::Ready,
                timestamp: Utc::now(),
            },
        ),
        Err(err) => {
            tracing::warn!(error = %err, "readiness check failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                ProbeResult {
                    status: ProbeStatus::NotReady,
                    timestamp: Utc::now(),
                },
            )
        }
    }
}

pub async fn status(state: &AppState) -> (StatusCode, StatusResult) {
    let cfg = &state.config;

    let (health, detail) = match state.db.ping().await {
        Ok(()) => (ComponentHealth::Healthy, None),
        Err(err) => (ComponentHealth::Unhealthy, Some(err.to_string())),
    };

    let overall = health;
    let http_status = match overall {
        ComponentHealth::Healthy => StatusCode::OK,
        ComponentHealth::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    (
        http_status,
        StatusResult {
            overall,
            service: cfg.primary.name.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: cfg.primary.env.clone(),
            timestamp: Utc::now(),
            database: ComponentStatus { health, detail },
        },
    )
}

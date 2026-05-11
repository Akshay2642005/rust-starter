use crate::domain::health;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ProbeResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentStatus {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusChecks {
    pub database: ComponentStatus,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub environment: String,
    pub timestamp: DateTime<Utc>,
    pub checks: StatusChecks,
}

impl From<health::ProbeResult> for ProbeResponse {
    fn from(r: health::ProbeResult) -> Self {
        let status = match r.status {
            health::ProbeStatus::Ok => "ok",
            health::ProbeStatus::Ready => "ready",
            health::ProbeStatus::NotReady => "not_ready",
        };
        Self {
            status: status.to_string(),
            timestamp: r.timestamp,
        }
    }
}

impl From<health::StatusResult> for StatusResponse {
    fn from(r: health::StatusResult) -> Self {
        let overall = match r.overall {
            health::ComponentHealth::Healthy => "healthy",
            health::ComponentHealth::Unhealthy => "degraded",
        };
        let db_status = match r.database.health {
            health::ComponentHealth::Healthy => "healthy",
            health::ComponentHealth::Unhealthy => "unhealthy",
        };
        Self {
            status: overall.to_string(),
            service: r.service,
            version: r.version,
            environment: r.environment,
            timestamp: r.timestamp,
            checks: StatusChecks {
                database: ComponentStatus {
                    status: db_status.to_string(),
                    detail: r.database.detail,
                },
            },
        }
    }
}

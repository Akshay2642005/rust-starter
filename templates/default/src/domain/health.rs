//! Health check domain types.

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeStatus {
    Ok,
    Ready,
    NotReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentHealth {
    Healthy,
    Unhealthy,
}

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub status: ProbeStatus,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ComponentStatus {
    pub health: ComponentHealth,
    pub detail: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StatusResult {
    pub overall: ComponentHealth,
    pub service: String,
    pub version: String,
    pub environment: String,
    pub timestamp: DateTime<Utc>,
    pub database: ComponentStatus,
}

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub primary: PrimaryConfig,
    pub store: DatabaseConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrimaryConfig {
    pub env: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    pub acquire_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
    pub sqlx_logging: bool,
}

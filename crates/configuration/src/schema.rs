use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub primary: PrimaryConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrimaryConfig {
    pub env: String,
    pub name: String,
}

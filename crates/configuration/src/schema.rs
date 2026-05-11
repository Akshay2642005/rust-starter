use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub primary: PrimaryConfig,
    pub store: DatabaseConfig,
    pub server: ServerConfig,
    pub telemetry: TelemetryConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrimaryConfig {
    pub env: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_read_timeout_secs")]
    pub read_timeout_secs: u64,
    #[serde(default = "default_write_timeout_secs")]
    pub write_timeout_secs: u64,
    #[serde(default = "default_idle_timeout_secs")]
    pub idle_timeout_secs: u64,
    #[serde(default = "default_request_timeout_secs")]
    pub request_timeout_secs: u64,
    #[serde(default = "default_shutdown_timeout_secs")]
    pub shutdown_timeout_secs: u64,
    #[serde(default = "default_min_graceful_shutdown_secs")]
    pub min_graceful_shutdown_secs: u64,
    #[serde(default = "default_max_body_size_bytes")]
    pub max_body_size_bytes: usize,
    pub cors_allowed_origins: Vec<String>,
    pub path_prefix: String,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum LogFormat {
    #[default]
    Compact,
    Pretty,
    Json,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelemetryConfig {
    #[serde(default = "default_service_name")]
    pub service_name: String,
    #[serde(default = "default_environment")]
    pub environment: String,
    #[serde(default = "default_filter")]
    pub filter: String,
    #[serde(default)]
    pub format: LogFormat,
    #[serde(default = "default_ansi")]
    pub ansi: bool,
    #[serde(default)]
    pub include_file: bool,
    #[serde(default)]
    pub include_line_number: bool,
    #[serde(default)]
    pub otlp: OtlpConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OtlpConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_endpoint")]
    pub endpoint: String,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            service_name: default_service_name(),
            environment: default_environment(),
            filter: default_filter(),
            format: LogFormat::default(),
            ansi: default_ansi(),
            include_file: false,
            include_line_number: false,
            otlp: OtlpConfig::default(),
        }
    }
}

impl Default for OtlpConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            endpoint: default_endpoint(),
        }
    }
}

fn default_service_name() -> String {
    "app".to_owned()
}

fn default_environment() -> String {
    "development".to_owned()
}

fn default_filter() -> String {
    "info,tower_http=info".to_owned()
}

fn default_ansi() -> bool {
    std::io::IsTerminal::is_terminal(&std::io::stderr())
}

fn default_enabled() -> bool {
    false
}

fn default_endpoint() -> String {
    "http://localhost:4317".to_owned()
}

fn default_host() -> String {
    "0.0.0.0".to_owned()
}

fn default_port() -> u16 {
    8080
}

fn default_read_timeout_secs() -> u64 {
    15
}
fn default_write_timeout_secs() -> u64 {
    15
}
fn default_idle_timeout_secs() -> u64 {
    60
}
fn default_request_timeout_secs() -> u64 {
    15
}

fn default_shutdown_timeout_secs() -> u64 {
    30
}

fn default_min_graceful_shutdown_secs() -> u64 {
    0
}

fn default_max_body_size_bytes() -> usize {
    10 * 1024 * 1024 // 10 MB
}

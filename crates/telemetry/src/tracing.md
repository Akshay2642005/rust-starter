```rust

use serde::Deserialize;
use tracing_subscriber::{EnvFilter, Layer, fmt, prelude::*};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogFormat {
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
}

impl Default for LogFormat {
    fn default() -> Self {
        Self::Compact
    }
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
        }
    }
}

#[derive(Debug)]
pub enum TraceInitError {
    InvalidFilter(tracing_subscriber::filter::ParseError),
    InstallSubscriber(tracing_subscriber::util::TryInitError),
}

impl std::fmt::Display for TraceInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFilter(error) => write!(f, "invalid telemetry filter: {error}"),
            Self::InstallSubscriber(error) => {
                write!(f, "failed to install tracing subscriber: {error}")
            }
        }
    }
}

impl std::error::Error for TraceInitError {}

pub fn init_tracing(config: TelemetryConfig) -> Result<(), TraceInitError> {
    let filter = EnvFilter::try_new(&config.filter).map_err(TraceInitError::InvalidFilter)?;

    let result = match &config.format {
        LogFormat::Compact => {
            let layer = base_fmt_layer(&config).compact().with_filter(filter);
            tracing_subscriber::registry()
                .with(layer)
                .try_init()
                .map_err(TraceInitError::InstallSubscriber)
        }
        LogFormat::Pretty => {
            let layer = base_fmt_layer(&config).pretty().with_filter(filter);
            tracing_subscriber::registry()
                .with(layer)
                .try_init()
                .map_err(TraceInitError::InstallSubscriber)
        }
        LogFormat::Json => {
            let layer = base_fmt_layer(&config)
                .json()
                .flatten_event(true)
                .with_current_span(true)
                .with_span_list(true)
                .with_filter(filter);

            tracing_subscriber::registry()
                .with(layer)
                .try_init()
                .map_err(TraceInitError::InstallSubscriber)
        }
    };

    if result.is_ok() {
        tracing::info!(
            service.name = %config.service_name,
            deployment.environment = %config.environment,
            telemetry.format = ?config.format,
            telemetry.filter = %config.filter,
            "telemetry initialized"
        );
    }

    result
}

fn base_fmt_layer<S>(config: &TelemetryConfig) -> fmt::Layer<S>
where
    S: tracing::Subscriber,
    S: for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    fmt::layer()
        .with_ansi(config.ansi)
        .with_target(true)
        .with_file(config.include_file)
        .with_line_number(config.include_line_number)
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

```

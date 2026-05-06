use std::sync::Arc;

use crate::error::TraceInitError;
use configuration::{Config, LogFormat};
use tracing_subscriber::{EnvFilter, Layer, fmt, prelude::*};

pub fn init_tracing(config: Arc<Config>) -> Result<(), TraceInitError> {
    let filter =
        EnvFilter::try_new(&config.telemetry.filter).map_err(TraceInitError::InvalidFilter)?;

    let result = match &config.telemetry.format {
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
            service.name = %config.telemetry.service_name,
            deployment.environment = %config.telemetry.environment,
            telemetry.format = ?config.telemetry.format,
            telemetry.filter = %config.telemetry.filter,
            "telemetry initialized"
        );
    }

    result
}

fn base_fmt_layer<S>(c: &Config) -> fmt::Layer<S>
where
    S: tracing::Subscriber,
    S: for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    fmt::layer()
        .with_ansi(c.telemetry.ansi)
        .with_target(true)
        .with_file(c.telemetry.include_file)
        .with_line_number(c.telemetry.include_line_number)
}

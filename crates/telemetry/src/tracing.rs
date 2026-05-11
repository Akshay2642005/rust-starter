use std::sync::Arc;

use crate::error::TraceInitError;
use configuration::{Config, LogFormat};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use tracing::Subscriber;
use tracing_error::ErrorLayer;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{EnvFilter, Layer, fmt, prelude::*};

pub struct TelemetryGuard;

pub fn init_tracing(config: Arc<Config>) -> Result<TelemetryGuard, TraceInitError> {
    let _ = tracing_log::LogTracer::init();

    let filter =
        EnvFilter::try_new(&config.telemetry.filter).map_err(TraceInitError::InvalidFilter)?;
    let fmt_layer = build_fmt_layer(&config, filter);

    if config.telemetry.otlp.enabled {
        let resource = Resource::new(vec![
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                config.telemetry.service_name.clone(),
            ),
            KeyValue::new(
                "deployment.environment",
                config.telemetry.environment.clone(),
            ),
        ]);

        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(&config.telemetry.otlp.endpoint)
            .build()
            .map_err(TraceInitError::OtlpPipeline)?;

        let provider = opentelemetry_sdk::trace::TracerProvider::builder()
            .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
            .with_resource(resource)
            .build();

        let tracer = opentelemetry::trace::TracerProvider::tracer(
            &provider,
            config.telemetry.service_name.clone(),
        );
        opentelemetry::global::set_tracer_provider(provider);

        let otlp_filter = EnvFilter::try_new(&config.telemetry.filter)
            .map_err(TraceInitError::InvalidFilter)?
            .add_directive("h2=off".parse().unwrap())
            .add_directive("hyper=off".parse().unwrap())
            .add_directive("tonic=off".parse().unwrap());

        install(
            tracing_subscriber::registry()
                .with(ErrorLayer::default())
                .with(fmt_layer)
                .with(
                    tracing_opentelemetry::layer()
                        .with_tracer(tracer)
                        .with_filter(otlp_filter),
                ),
        )?;
    } else {
        install(
            tracing_subscriber::registry()
                .with(ErrorLayer::default())
                .with(fmt_layer),
        )?;
    }

    tracing::info!(
        service.name = %config.telemetry.service_name,
        deployment.environment = %config.telemetry.environment,
        "telemetry initialized"
    );

    Ok(TelemetryGuard)
}

fn install<S>(subscriber: S) -> Result<(), TraceInitError>
where
    S: Subscriber + Send + Sync + 'static,
{
    tracing::dispatcher::set_global_default(tracing::Dispatch::new(subscriber))
        .map_err(TraceInitError::InstallSubscriber)
}

fn build_fmt_layer<S>(c: &Config, filter: EnvFilter) -> Box<dyn Layer<S> + Send + Sync>
where
    S: tracing::Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    match &c.telemetry.format {
        LogFormat::Compact => base_fmt_layer(c).compact().with_filter(filter).boxed(),
        LogFormat::Pretty => base_fmt_layer(c).pretty().with_filter(filter).boxed(),
        LogFormat::Json => base_fmt_layer(c)
            .json()
            .flatten_event(true)
            .with_current_span(true)
            .with_span_list(true)
            .with_filter(filter)
            .boxed(),
    }
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
        .with_span_events(FmtSpan::CLOSE)
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        tracing::info!("telemetry shutting down");
        opentelemetry::global::shutdown_tracer_provider();
    }
}

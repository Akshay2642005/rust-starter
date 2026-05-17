#[derive(Debug)]
pub enum TraceInitError {
    InvalidFilter(tracing_subscriber::filter::ParseError),
    InstallSubscriber(tracing::dispatcher::SetGlobalDefaultError),
    LogBridge(log::SetLoggerError),
    OtlpPipeline(opentelemetry::trace::TraceError),
}

impl std::fmt::Display for TraceInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFilter(e) => write!(f, "invalid telemetry filter: {e}"),
            Self::InstallSubscriber(e) => write!(f, "failed to install telemetry subscriber: {e}"),
            Self::LogBridge(e) => write!(f, "failed to initialize log bridge: {e}"),
            Self::OtlpPipeline(e) => write!(f, "failed to build OTLP pipeline: {e}"),
        }
    }
}

impl std::error::Error for TraceInitError {}

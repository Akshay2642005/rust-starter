#[derive(Debug)]
pub enum TraceInitError {
    InvalidFilter(tracing_subscriber::filter::ParseError),
    InstallSubscriber(tracing_subscriber::util::TryInitError),
    LogBridge(log::SetLoggerError),
    OtlpPipeline(opentelemetry::trace::TraceError),
}

impl std::fmt::Display for TraceInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFilter(error) => write!(f, "invalid telemetry filter: {error}"),
            Self::InstallSubscriber(error) => {
                write!(f, "failed to install telemetry subscriber: {error}")
            }
            Self::LogBridge(error) => write!(f, "failed to initialize log bridge: {error}"),
            Self::OtlpPipeline(error) => write!(f, "failed to build OTLP pipeline: {error}"),
        }
    }
}

impl std::error::Error for TraceInitError {}

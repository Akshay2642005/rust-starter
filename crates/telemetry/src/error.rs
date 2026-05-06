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
                write!(f, "failed to install telemetry subscriber: {error}")
            }
        }
    }
}

impl std::error::Error for TraceInitError {}

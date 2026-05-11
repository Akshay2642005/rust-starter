mod error;
mod tracing;
pub use error::TraceInitError;
pub use tracing::{TelemetryGuard, init_tracing};

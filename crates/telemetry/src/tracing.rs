use tracing_subscriber::{EnvFilter, prelude::*};

pub fn init_tracing() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".parse().unwrap()));

    tracing_subscriber::registry().with(fmt_layer).init();
}

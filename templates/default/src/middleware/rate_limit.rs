//! Rate limiting layer for sensitive endpoints (e.g. auth).
//!
//! Burst of 10 requests, refilling 1 token per 2 seconds per IP.

use governor::middleware::NoOpMiddleware;
use std::sync::Arc;
use tower_governor::governor::{GovernorConfig, GovernorConfigBuilder};
use tower_governor::key_extractor::PeerIpKeyExtractor;

pub use tower_governor::GovernorLayer;

pub fn auth_rate_limit_layer() -> Arc<GovernorConfig<PeerIpKeyExtractor, NoOpMiddleware>> {
    Arc::new(
        GovernorConfigBuilder::default()
            .per_millisecond(2_000)
            .burst_size(10)
            .finish()
            .expect("invalid rate limit config"),
    )
}

pub fn global_rate_limit_layer() -> Arc<GovernorConfig<PeerIpKeyExtractor, NoOpMiddleware>> {
    Arc::new(
        GovernorConfigBuilder::default()
            // more relaxed than auth
            .per_second(10)
            .burst_size(50)
            .finish()
            .expect("invalid rate limit config"),
    )
}

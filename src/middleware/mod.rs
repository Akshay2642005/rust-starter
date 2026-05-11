//! HTTP middleware composition for the API router.
//!
//! This module builds a layered middleware stack for security, tracing,
//! CORS, request IDs, body limits, and timeouts.
mod cors;
mod request_id;
mod security;
mod timeout;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{Request, Response},
};
use configuration::Config;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    sensitive_headers::SetSensitiveRequestHeadersLayer,
};

/// Applies standard middleware layers to the provided router.
///
/// Uses configuration to derive CORS, limits, and timeouts.
pub fn apply(router: Router, config: &Config) -> Router {
    let request_id_header = request_id::request_id_header_name();
    let cors_layer = cors::build_cors_layer(config);
    let max_body_size = config.server.max_body_size_bytes;
    let request_timeout_secs = config.server.request_timeout_secs;

    let service_stack = ServiceBuilder::new()
        .layer(CatchPanicLayer::new())
        .layer(CompressionLayer::new())
        .layer(security::cache_control_layer())
        .layer(security::content_type_options_layer())
        .layer(security::frame_options_layer())
        .layer(security::referrer_policy_layer())
        .layer(SetSensitiveRequestHeadersLayer::new([
            axum::http::header::AUTHORIZATION,
            axum::http::header::COOKIE,
        ]))
        .layer(RequestBodyLimitLayer::new(max_body_size))
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let request_id = request_id::request_id_from_headers(request.headers())
                        .unwrap_or_else(|| "unknown".to_string());

                    tracing::info_span!(
                        "http_request",
                        request_id = %request_id,
                        method = %request.method(),
                        path = %request.uri().path(),
                        status = tracing::field::Empty,
                    )
                })
                .on_response(
                    |response: &Response<_>, latency: Duration, span: &tracing::Span| {
                        let status = response.status();
                        span.record("status", status.as_u16());

                        if status.is_server_error() {
                            tracing::error!(
                                parent: span,
                                status = status.as_u16(),
                                latency_ms = latency.as_millis() as u64,
                                "request completed with server error"
                            );
                        } else if status.is_client_error() {
                            tracing::warn!(
                                parent: span,
                                status = status.as_u16(),
                                latency_ms = latency.as_millis() as u64,
                                "request completed with client error"
                            );
                        } else {
                            tracing::info!(
                                parent: span,
                                status = status.as_u16(),
                                latency_ms = latency.as_millis() as u64,
                                "request completed"
                            );
                        }
                    },
                ),
        )
        .layer(PropagateRequestIdLayer::new(request_id_header.clone()))
        .layer(SetRequestIdLayer::new(request_id_header, MakeRequestUuid))
        .layer(cors_layer);

    router
        .layer(DefaultBodyLimit::max(max_body_size))
        .route_layer(axum::middleware::from_fn(move |request, next| {
            timeout::timeout_middleware(request, next, request_timeout_secs)
        }))
        .layer(service_stack)
}

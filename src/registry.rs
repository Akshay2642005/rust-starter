//! Route registry and installation utilities for HTTP handlers.
use crate::state::AppState;
use axum::Router;
use std::sync::OnceLock;

pub type RouteInstaller = fn(Router<AppState>) -> Router<AppState>;

pub struct RouteEntry {
    pub install: RouteInstaller,
}

impl RouteEntry {
    pub const fn new(install: RouteInstaller) -> Self {
        Self { install }
    }
}

inventory::collect!(RouteEntry);

static ROUTE_PREFIX: OnceLock<String> = OnceLock::new();

const ROOT_SCOPED_PATHS: &[&str] = &["/health", "/healthz", "/livez", "/readyz", "/status"];

pub fn set_route_prefix(prefix: impl Into<String>) {
    let prefix = prefix.into();

    if let Err(_existing) = ROUTE_PREFIX.set(prefix) {
        tracing::debug!("route prefix already initialized");
    }
}

#[must_use]
pub fn scoped_path(path: &str) -> String {
    if ROOT_SCOPED_PATHS.contains(&path) {
        return path.to_string();
    }

    let prefix = ROUTE_PREFIX.get().map(String::as_str).unwrap_or_default();

    if prefix.is_empty() || prefix == "/" {
        return path.to_string();
    }

    format!("{}{}", normalize_prefix(prefix), path)
}

pub fn install_routes(mut router: Router<AppState>) -> Router<AppState> {
    for entry in inventory::iter::<RouteEntry> {
        router = (entry.install)(router);
    }

    router
}

fn normalize_prefix(prefix: &str) -> &str {
    prefix.trim_end_matches('/')
}

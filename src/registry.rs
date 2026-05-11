//! Route registry and installation utilities for HTTP handlers.
use crate::state::AppState;
use axum::Router;

pub type RouteInstaller = fn(Router<AppState>, &str) -> Router<AppState>;

pub struct RouteEntry {
    pub install: RouteInstaller,
}

impl RouteEntry {
    pub const fn new(install: RouteInstaller) -> Self {
        Self { install }
    }
}

inventory::collect!(RouteEntry);

const ROOT_SCOPED_PATHS: &[&str] = &["/health", "/healthz", "/livez", "/readyz", "/status"];

#[must_use]
pub fn scoped_path(path: &str, prefix: &str) -> String {
    if ROOT_SCOPED_PATHS.contains(&path) {
        return path.to_string();
    }

    join_paths(prefix, path)
}

#[must_use]
pub fn join_paths(prefix: &str, path: &str) -> String {
    let prefix = normalize_path_segment(prefix);
    let path = normalize_path_segment(path);

    if prefix.is_empty() {
        return format!("/{path}");
    }

    if path.is_empty() {
        return format!("/{prefix}");
    }

    format!("/{prefix}/{path}")
}

pub fn install_routes(mut router: Router<AppState>, prefix: &str) -> Router<AppState> {
    for entry in inventory::iter::<RouteEntry> {
        router = (entry.install)(router, prefix);
    }

    router
}

fn normalize_path_segment(path: &str) -> &str {
    path.trim_matches('/')
}

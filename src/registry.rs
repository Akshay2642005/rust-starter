//! Route registry and installation utilities for HTTP handlers.

use axum::{Router, middleware};

use crate::{middleware::require_auth, state::AppState};

pub type RouteInstaller = fn(Router<AppState>, &str) -> Router<AppState>;

pub struct RouteEntry {
    pub install: RouteInstaller,
    pub protected: bool,
}

impl RouteEntry {
    pub const fn new(install: RouteInstaller, protected: bool) -> Self {
        Self { install, protected }
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

pub fn install_routes(router: Router<AppState>, prefix: &str, state: AppState) -> Router<AppState> {
    let mut public_router = Router::new();

    let mut protected_router = Router::new();

    for entry in inventory::iter::<RouteEntry> {
        if entry.protected {
            protected_router = (entry.install)(protected_router, prefix);
        } else {
            public_router = (entry.install)(public_router, prefix);
        }
    }

    protected_router =
        protected_router.route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    router.merge(public_router).merge(protected_router)
}

fn normalize_path_segment(path: &str) -> &str {
    path.trim_matches('/')
}


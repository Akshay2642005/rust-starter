//! Procedural macros for route registration and handler instrumentation.

mod db;
mod instrument;
mod route;
mod shutdown;

use proc_macro::TokenStream;

/// Registers an Axum route and submits it to the route registry.
///
/// Usage: `#[route(GET, "/path")]`.
#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    route::expand(attr, item)
}

/// Adds tracing instrumentation to async handlers.
///
/// This macro validates that the target function is async.
#[proc_macro_attribute]
pub fn instrument_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    instrument::expand(item)
}

/// Adds database tracing instrumentation with statement metadata.
#[proc_macro_attribute]
pub fn instrument_db(attr: TokenStream, item: TokenStream) -> TokenStream {
    db::expand(attr, item)
}

/// Generates a cross-platform graceful shutdown signal handler.
#[proc_macro_attribute]
pub fn graceful_shutdown(_attr: TokenStream, item: TokenStream) -> TokenStream {
    shutdown::expand(item)
}

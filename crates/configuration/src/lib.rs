/*
     @ Configuration crate
        - Uses the `config` crate to manage application configuration. along with 'arc-swap' for thread-safe configuration updates.
        - Provides a `ConfigManager` struct that loads configuration from a file and allows for dynamic
        - updates to the configuration at runtime. The configuration is stored in an `ArcSwap` to ensure thread safety when accessed by multiple threads.
*/

mod handle;
mod loader;
mod schema;
mod validate;
mod watcher;

pub(crate) use handle::ConfigHandle;
pub use loader::load_config;
pub use schema::{Config, PrimaryConfig};
pub use watcher::ConfigManager;


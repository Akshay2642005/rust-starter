mod app;
mod domain;
mod handlers;
mod middleware;
mod registry;
mod response;
mod services;
mod state;

pub use state::AppState;

use std::time::Duration;

use anyhow::Context;
use configuration::ConfigManager;
use macros::graceful_shutdown;
use tokio::sync::broadcast;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());
    let config_dir = std::env::var("CONFIG_DIR").unwrap_or_else(|_| ".".into());

    let (manager, config_handle) =
        ConfigManager::load_initial(&app_env, &config_dir).context("failed to load config")?;
    let config = config_handle.load();

    let (shutdown_tx, _) = broadcast::channel::<()>(2);

    let shutdown_task = {
        let shutdown_tx = shutdown_tx.clone();
        tokio::spawn(async move {
            shutdown_signal().await;
            let _ = shutdown_tx.send(());
        })
    };

    let watcher_handle = manager.spawn_watcher(shutdown_tx.subscribe());
    let _telemetry =
        telemetry::init_tracing(config.clone()).context("failed to initialize telemetry")?;

    info!(env = %app_env, name = %config.primary.name, "config loaded");
    let server = app::ServerBuilder::new(config).build().await?;
    let server_result = server.run(shutdown_tx.subscribe()).await;

    let _ = shutdown_tx.send(());

    let _ = tokio::time::timeout(Duration::from_secs(5), watcher_handle).await;

    shutdown_task.abort();

    server_result?;
    Ok(())
}

#[graceful_shutdown]
async fn shutdown_signal() {}

mod app;
mod state;

use anyhow::Context;
use configuration::ConfigManager;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());
    let config_dir = std::env::var("CONFIG_DIR").unwrap_or_else(|_| ".".into());

    let (manager, config_handle) =
        ConfigManager::load_initial(&app_env, &config_dir).context("failed to load config")?;
    let config = config_handle.load();

    manager.spawn_watcher();
    let _telemetry = telemetry::init_tracing(config.clone()).context("failed to initialize telemetry")?;

    info!(env = %app_env, name = %config.primary.name, "config loaded");
    let server = app::ServerBuilder::new(config).build().await?;
    server.run().await?;

    Ok(())
}

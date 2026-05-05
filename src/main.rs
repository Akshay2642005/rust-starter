#![allow(dead_code)]
mod app;
mod state;

use anyhow::Context;
use configuration::ConfigManager;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".parse().unwrap()),
        )
        .init();

    let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());

    let config_dir = match app_env.as_str() {
        "production" => {
            let app_name = std::env::var("APP_NAME").expect("APP_NAME must be set in production");

            format!("/etc/{}/config", app_name)
        }
        _ => "config".to_string(),
    };

    let (_, config_handle) =
        ConfigManager::load_initial(&app_env, &config_dir).context("failed to load config")?;

    let config = config_handle.load();

    info!(env = %app_env, name = %config.primary.name, "config loaded");

    let server = app::ServerBuilder::new(config).build().await?;
    server.run().await?;

    Ok(())
}

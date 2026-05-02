use configuration::ConfigManager;

fn main() -> anyhow::Result<()> {
    let env_name = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    let (_, config_handle) = ConfigManager::load_initial(&env_name, "config")?;
    let config = config_handle.load();

    println!("Current environment: {}", env_name);
    println!("Configuration: {:#?}", config);
    Ok(())
}

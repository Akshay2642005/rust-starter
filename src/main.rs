use configuration::ConfigManager;
use seaorm::{AppSchema, SeaOrmStore};

pub struct Schema;
impl AppSchema for Schema {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env_name = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    let (_, config_handle) = ConfigManager::load_initial(&env_name, "config")?;
    let config = config_handle.load();

    println!("Current environment: {}", env_name);
    println!("Configuration: {:#?}", config);
    println!("Connecting to database at: {}", config.store.url);

    let store = seaorm::SeaOrmStore::<Schema>::connect_and_migrate(&config).await?;
    SeaOrmStore::ping(&store).await?;

    Ok(())
}

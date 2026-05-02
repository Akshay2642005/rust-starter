use config::{Config as C, Environment, File};
use std::path::Path;

use crate::{schema::Config, validate::validate_configuration};
pub fn load_config(env_name: &str, config_dir: &Path) -> anyhow::Result<Config> {
    let _ = dotenvy::dotenv();

    let builder = C::builder()
        .add_source(File::with_name(
            &config_dir.join("default").to_string_lossy(),
        ))
        .add_source(File::with_name(&config_dir.join(env_name).to_string_lossy()).required(false))
        .add_source(File::with_name(&config_dir.join("local").to_string_lossy()).required(false))
        .add_source(
            Environment::with_prefix("APP")
                .separator("__")
                .try_parsing(true),
        );

    let config = builder.build()?.try_deserialize::<Config>()?;
    validate_configuration(&config)?;
    Ok(config)
}

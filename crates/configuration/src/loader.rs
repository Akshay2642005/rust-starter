use config::{Config as C, ConfigBuilder, Environment, File, FileFormat};
use std::path::Path;

use crate::{schema::Config, validate::validate_configuration};

fn add_base_config(
    builder: ConfigBuilder<config::builder::DefaultState>,
    config_dir: &Path,
) -> ConfigBuilder<config::builder::DefaultState> {
    let yaml = config_dir.join("config.yml");
    let toml = config_dir.join("config.toml");

    if yaml.exists() {
        builder.add_source(File::from(yaml).format(FileFormat::Yaml).required(true))
    } else if toml.exists() {
        builder.add_source(File::from(toml).format(FileFormat::Toml).required(true))
    } else {
        panic!("No default config file found (expected config.yml or config.toml)");
    }
}

fn add_optional_config(
    builder: ConfigBuilder<config::builder::DefaultState>,
    config_dir: &Path,
    name: &str,
) -> ConfigBuilder<config::builder::DefaultState> {
    let yaml = config_dir.join(format!("{name}.yml"));
    let toml = config_dir.join(format!("{name}.toml"));

    if yaml.exists() {
        builder.add_source(File::from(yaml).format(FileFormat::Yaml).required(false))
    } else if toml.exists() {
        builder.add_source(File::from(toml).format(FileFormat::Toml).required(false))
    } else {
        builder
    }
}

pub fn load_config(env_name: &str, config_dir: &Path) -> anyhow::Result<Config> {
    let _ = dotenvy::dotenv();

    let builder = C::builder();

    let builder = add_base_config(builder, config_dir);
    let builder = add_optional_config(builder, config_dir, env_name);
    let builder = add_optional_config(builder, config_dir, "local");

    let builder = builder.add_source(
        Environment::with_prefix("APP")
            .separator("__")
            .try_parsing(true),
    );

    let config = builder.build()?.try_deserialize::<Config>()?;

    validate_configuration(&config)?;
    Ok(config)
}

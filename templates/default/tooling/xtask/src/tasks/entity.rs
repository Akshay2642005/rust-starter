use std::{fs, path::Path};

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use xshell::{Shell, cmd};

const DEFAULT_ENTITY_OUTPUT: &str = "crates/seaorm/src/store/entities";
const IGNORED_TABLES: &str = "seaql_migrations,_schema_migrations";

#[derive(Args)]
pub struct EntityArgs {
    #[command(subcommand)]
    command: EntityCommand,
}

#[derive(Subcommand)]
enum EntityCommand {
    /// Generate SeaORM entities into crates/seaorm/src/store/entities.
    Generate {
        /// Database URL. Falls back to DATABASE_URL.
        #[arg(short, long, env = "DATABASE_URL")]
        database_url: Option<String>,
        /// Output directory for generated entities.
        #[arg(short, long, default_value = DEFAULT_ENTITY_OUTPUT)]
        output: String,
    },
}

pub fn run(sh: &Shell, args: EntityArgs) -> Result<()> {
    match args.command {
        EntityCommand::Generate {
            database_url,
            output,
        } => {
            let database_url = database_url
                .context("missing database URL; pass --database-url or set DATABASE_URL")?;
            generate(sh, &database_url, &output)?;
        }
    }

    Ok(())
}

pub fn default_output() -> &'static str {
    DEFAULT_ENTITY_OUTPUT
}

pub fn generate(sh: &Shell, database_url: &str, output: &str) -> Result<()> {
    cmd!(
        sh,
        "sea-orm-cli generate entity -u {database_url} -o {output} --ignore-tables {IGNORED_TABLES} --with-serde both --date-time-crate chrono"
    )
    .run()?;

    remove_migration_entities(output)?;

    Ok(())
}

fn remove_migration_entities(output: &str) -> Result<()> {
    let output = Path::new(output);
    let migration_modules = [
        "_schema_migrations",
        "schema_migrations",
        "seaql_migrations",
        "migrations",
    ];

    for module in migration_modules {
        let path = output.join(format!("{module}.rs"));
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("failed to remove {}", path.display()))?;
        }
    }

    remove_module_exports(&output.join("mod.rs"), &migration_modules)?;
    remove_module_exports(&output.join("prelude.rs"), &migration_modules)?;

    Ok(())
}

fn remove_module_exports(path: &Path, modules: &[&str]) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let filtered = contents
        .lines()
        .filter(|line| {
            !modules
                .iter()
                .any(|module| line.contains(module) || line.contains(&pascal_case(module)))
        })
        .collect::<Vec<_>>()
        .join("\n");

    let filtered = format!("{filtered}\n");
    if filtered != contents {
        fs::write(path, filtered)
            .with_context(|| format!("failed to update {}", path.display()))?;
    }

    Ok(())
}

fn pascal_case(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

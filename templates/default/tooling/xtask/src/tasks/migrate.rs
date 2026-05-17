use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use chrono::Local;
use clap::{Args, Subcommand};
use sea_orm::{
    ConnectionTrait, Database, DbBackend, Statement, TransactionTrait, sea_query::Value,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use xshell::Shell;

use crate::tasks::entity;

const MIGRATIONS_TABLE: &str = "_schema_migrations";

#[derive(Args)]
pub struct GenerateSqlArgs {
    /// Migration name, for example `create_users`.
    pub name: String,
}

#[derive(Args)]
pub struct MigrateGenerateArgs {
    /// Database URL. Falls back to DATABASE_URL, then config.yml store.url.
    #[arg(short, long, env = "DATABASE_URL")]
    database_url: Option<String>,
    /// Output directory for generated entities.
    #[arg(short, long, default_value = entity::default_output())]
    output: String,
}

#[derive(Args)]
pub struct MigrateArgs {
    /// Database URL. Falls back to DATABASE_URL, then config.yml store.url.
    #[arg(short, long, env = "DATABASE_URL")]
    database_url: Option<String>,
    #[command(subcommand)]
    command: MigrateCommand,
}

#[derive(Subcommand)]
enum MigrateCommand {
    /// Create a timestamped SQL migration file.
    Generate { name: String },
    /// Apply pending SQL migrations.
    Up,
    /// Show applied and pending SQL migrations.
    Status,
    /// List SQL migration files.
    List,
}

pub fn migrate_generate(sh: &Shell, args: MigrateGenerateArgs) -> Result<()> {
    let database_url = database_url(args.database_url)?;
    runtime()?.block_on(apply_pending(&database_url))?;
    entity::generate(sh, &database_url, &args.output)?;
    Ok(())
}

pub fn run(_sh: &Shell, args: MigrateArgs) -> Result<()> {
    match args.command {
        MigrateCommand::Generate { name } => {
            create_migration(&name)?;
        }
        MigrateCommand::Up => {
            let database_url = database_url(args.database_url)?;
            runtime()?.block_on(apply_pending(&database_url))?;
        }
        MigrateCommand::Status => {
            let database_url = database_url(args.database_url)?;
            runtime()?.block_on(print_status(&database_url))?;
        }
        MigrateCommand::List => {
            list_migrations()?;
        }
    }

    Ok(())
}

async fn apply_pending(database_url: &str) -> Result<()> {
    let db = Database::connect(database_url)
        .await
        .context("failed to connect to database")?;
    ensure_migrations_table(&db).await?;

    let applied = applied_migrations(&db).await?;
    let migrations = migration_files()?;
    let mut applied_count = 0usize;

    for migration in migrations {
        let version = migration_version(&migration)?;
        if applied.contains(&version) {
            continue;
        }

        let sql = fs::read_to_string(&migration)
            .with_context(|| format!("failed to read {}", migration.display()))?;
        let checksum = checksum(&sql);
        let tx = db.begin().await.context("failed to begin transaction")?;

        tx.execute_unprepared(&sql)
            .await
            .with_context(|| format!("failed to apply {}", migration.display()))?;
        record_migration(&tx, &version, &checksum).await?;
        tx.commit().await.context("failed to commit migration")?;

        println!("applied {version}");
        applied_count += 1;
    }

    if applied_count == 0 {
        println!("no pending migrations");
    }

    Ok(())
}

async fn print_status(database_url: &str) -> Result<()> {
    let db = Database::connect(database_url)
        .await
        .context("failed to connect to database")?;
    ensure_migrations_table(&db).await?;

    let applied = applied_migrations(&db).await?;
    let migrations = migration_files()?;

    if migrations.is_empty() {
        println!("no migration files found");
        return Ok(());
    }

    for migration in migrations {
        let version = migration_version(&migration)?;
        let state = if applied.contains(&version) {
            "applied"
        } else {
            "pending"
        };
        println!("{state:8} {version}");
    }

    Ok(())
}

fn create_migration(name: &str) -> Result<()> {
    let migrations_dir = Path::new("migrations");
    fs::create_dir_all(migrations_dir)
        .with_context(|| format!("failed to create {}", migrations_dir.display()))?;

    let slug = slugify(name)?;
    let timestamp = Local::now().format("%Y%m%d%H%M%S");
    let path = migrations_dir.join(format!("{timestamp}_{slug}.sql"));

    write_new_file(&path, "-- Add migration script here\n")?;
    println!("generated {}", path.display());

    Ok(())
}

fn list_migrations() -> Result<()> {
    let entries = migration_files()?;

    if entries.is_empty() {
        println!("no migrations directory found");
        return Ok(());
    }

    for entry in entries {
        println!("{}", entry.display());
    }

    Ok(())
}

fn migration_files() -> Result<Vec<PathBuf>> {
    let migrations_dir = Path::new("migrations");
    if !migrations_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(migrations_dir)
        .with_context(|| format!("failed to read {}", migrations_dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("sql"))
        .collect::<Vec<_>>();

    entries.sort();
    Ok(entries)
}

fn slugify(name: &str) -> Result<String> {
    let slug = name
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_");

    if slug.is_empty() {
        bail!("migration name must contain at least one letter or number");
    }

    Ok(slug)
}

fn write_new_file(path: &Path, contents: &str) -> Result<()> {
    if path.exists() {
        bail!("refusing to overwrite existing file {}", path.display());
    }

    fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))
}

async fn ensure_migrations_table(db: &impl ConnectionTrait) -> Result<()> {
    db.execute_unprepared(&format!(
        r#"
CREATE TABLE IF NOT EXISTS {MIGRATIONS_TABLE} (
    version TEXT PRIMARY KEY,
    checksum TEXT NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
"#
    ))
    .await
    .context("failed to ensure migrations table")?;

    Ok(())
}

async fn applied_migrations(db: &impl ConnectionTrait) -> Result<BTreeSet<String>> {
    let rows = db
        .query_all(Statement::from_string(
            DbBackend::Postgres,
            format!("SELECT version FROM {MIGRATIONS_TABLE} ORDER BY version"),
        ))
        .await
        .context("failed to load applied migrations")?;

    rows.into_iter()
        .map(|row| row.try_get("", "version").context("failed to read version"))
        .collect()
}

async fn record_migration(db: &impl ConnectionTrait, version: &str, checksum: &str) -> Result<()> {
    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        format!("INSERT INTO {MIGRATIONS_TABLE} (version, checksum) VALUES ($1, $2)"),
        [Value::from(version), Value::from(checksum)],
    ))
    .await
    .with_context(|| format!("failed to record migration {version}"))?;

    Ok(())
}

fn migration_version(path: &Path) -> Result<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
        .context("migration path has a non-utf8 file name")
}

fn checksum(contents: &str) -> String {
    let digest = Sha256::digest(contents.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn database_url(value: Option<String>) -> Result<String> {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        return Ok(value);
    }

    config_database_url()
}

fn config_database_url() -> Result<String> {
    #[derive(Deserialize)]
    struct AppConfig {
        store: StoreConfig,
    }

    #[derive(Deserialize)]
    struct StoreConfig {
        url: String,
    }

    let cfg: AppConfig = config::Config::builder()
        .add_source(config::File::with_name("config").required(false))
        .build()
        .context("failed to load config.yml")?
        .try_deserialize()
        .context("missing database URL; pass --database-url, set DATABASE_URL, or configure store.url in config.yml")?;

    Ok(cfg.store.url)
}

fn runtime() -> Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime")
}

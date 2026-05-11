mod tasks;
mod workspace;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cargo xtask")]
#[command(about = "Project automation for rust-starter")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run formatting, clippy, and tests.
    Check,
    /// Generate SeaORM entities from the configured database.
    Entity(tasks::entity::EntityArgs),
    /// Apply migrations, then regenerate SeaORM entities.
    MigrateGenerate(tasks::migrate::MigrateGenerateArgs),
    /// Work with SQL migration files.
    Migrate(tasks::migrate::MigrateArgs),
    /// Run benchmarks.
    Bench,
    /// Run workspace tests.
    Test,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let sh = workspace::shell()?;

    match args.command {
        Command::Check => tasks::check::run(&sh),
        Command::Entity(args) => tasks::entity::run(&sh, args),
        Command::MigrateGenerate(args) => tasks::migrate::migrate_generate(&sh, args),
        Command::Migrate(args) => tasks::migrate::run(&sh, args),
        Command::Bench => tasks::bench::run(&sh),
        Command::Test => tasks::test::run(&sh),
    }
}

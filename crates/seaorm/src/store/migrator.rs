//! Application migration registry.
//!
//! Uses a custom `app_migrations` table so it does not conflict with
//! `better-auth-rs`'s `better_auth_migrations` table or the default
//! `seaql_migrations` table produced by SeaORM's own tools.
//!
//! # Adding a migration
//!
//! 1. Create a new migration struct that implements `MigrationTrait`.
//! 2. Push `Box::new(YourMigration)` to the `migrations()` vector below,
//!    **in chronological order**.
//!
//! Migrations are run automatically on startup via
//! `SeaOrmStore::connect_and_migrate`.

use sea_orm::sea_query::IntoIden;
use sea_orm_migration::prelude::*;

use crate::store::migrations;

// Import concrete migration structs here as you add them.
// mod m20240101_000001_create_workspaces;

pub struct AppMigrator;

#[async_trait::async_trait]
impl MigratorTrait for AppMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // Add new migrations here in order.
            Box::new(migrations::m20260503_170604_create_table::Migration),
        ]
    }

    /// Separate history table — never collides with better-auth or SeaORM CLI.
    fn migration_table_name() -> sea_orm::DynIden {
        "app_migrations".into_iden()
    }
}

/// Convenience helper for tests.
pub async fn run_migrations(db: &sea_orm::DatabaseConnection) -> Result<(), DbErr> {
    AppMigrator::up(db, None).await
}

//! Generic SeaORM store.

mod entities;
mod migrations;
mod migrator;
mod repository;

// Example entity — shows how to wire up a real domain entity.

use std::time::Duration;

use sea_orm::{
    ConnectOptions, Database, DatabaseConnection, DatabaseTransaction, DbErr, TransactionTrait,
};
use tracing::info;

use crate::error::{OrmResult, map_db_err};
use crate::schema::AppSchema;

#[derive(Debug, Clone)]
pub struct StoreConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    pub acquire_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
    pub sqlx_logging: bool,
}

impl StoreConfig {
    pub fn from_env() -> Self {
        Self {
            url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            max_connections: env_u32("DB_MAX_CONNECTIONS", 10),
            min_connections: env_u32("DB_MIN_CONNECTIONS", 2),
            connect_timeout_secs: env_u64("DB_CONNECT_TIMEOUT_SECS", 10),
            acquire_timeout_secs: env_u64("DB_ACQUIRE_TIMEOUT_SECS", 5),
            idle_timeout_secs: env_u64("DB_IDLE_TIMEOUT_SECS", 600),
            max_lifetime_secs: env_u64("DB_MAX_LIFETIME_SECS", 1800),
            sqlx_logging: std::env::var("DB_SQLX_LOGGING")
                .map(|v| v == "true")
                .unwrap_or(false),
        }
    }
}

fn env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// The central store, generic over the application's `AppSchema` `S`.
///
/// `Clone` is cheap — `DatabaseConnection` holds an `Arc` internally.
///
/// # Construction
///
/// ```rust,ignore
/// let cfg   = StoreConfig::from_env();
/// let store = SeaOrmStore::<MySchema>::connect_and_migrate(&cfg).await?;
/// ```
#[derive(Clone)]
pub struct SeaOrmStore<S: AppSchema> {
    db: DatabaseConnection,
    // S is held for future extension (e.g. schema-specific middleware).
    _schema: std::marker::PhantomData<S>,
}

impl<S: AppSchema> SeaOrmStore<S> {
    /// Connect to Postgres and run pending migrations.
    pub async fn connect_and_migrate(cfg: &StoreConfig) -> OrmResult<Self> {
        use migrator::AppMigrator;
        use sea_orm_migration::MigratorTrait;

        let mut opts = ConnectOptions::new(&cfg.url);
        opts.max_connections(cfg.max_connections)
            .min_connections(cfg.min_connections)
            .connect_timeout(Duration::from_secs(cfg.connect_timeout_secs))
            .acquire_timeout(Duration::from_secs(cfg.acquire_timeout_secs))
            .idle_timeout(Duration::from_secs(cfg.idle_timeout_secs))
            .max_lifetime(Duration::from_secs(cfg.max_lifetime_secs))
            .sqlx_logging(cfg.sqlx_logging);

        info!("connecting to postgres…");
        let db = Database::connect(opts).await.map_err(map_db_err)?;

        info!("running pending migrations…");
        AppMigrator::up(&db, None).await.map_err(map_db_err)?;

        info!("store ready");
        Ok(Self {
            db,
            _schema: std::marker::PhantomData,
        })
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    // Alias used by entity-specific repos.
    pub(crate) fn db_ref(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Liveness / readiness check.
    pub async fn ping(&self) -> Result<(), DbErr> {
        self.db.ping().await
    }

    /// Execute `work` inside a single ACID transaction.
    /// Commits on `Ok(_)`, rolls back on `Err(_)`.
    ///
    /// ```rust,ignore
    /// store.transaction(|tx| Box::pin(async move {
    ///     workspace_repo.create_in_tx(tx, dto).await?;
    ///     member_repo.create_in_tx(tx, member_dto).await?;
    ///     Ok(())
    /// })).await?;
    /// ```
    pub async fn transaction<F, R>(&self, work: F) -> OrmResult<R>
    where
        F: for<'tx> FnOnce(
                &'tx DatabaseTransaction,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = OrmResult<R>> + Send + 'tx>,
            > + Send,
        R: Send,
    {
        let tx = self.db.begin().await.map_err(map_db_err)?;
        match work(&tx).await {
            Ok(v) => {
                tx.commit().await.map_err(map_db_err)?;
                Ok(v)
            }
            Err(e) => {
                tx.rollback().await.map_err(map_db_err)?;
                Err(e)
            }
        }
    }
}

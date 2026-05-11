//! Generic SeaORM store.

pub mod entities;
pub mod repository;

use configuration::Config;

use sea_orm::{
    ConnectOptions, Database, DatabaseConnection, DatabaseTransaction, DbErr, TransactionTrait,
};
use std::{sync::Arc, time::Duration};
use tracing::info;

use crate::error::{OrmResult, map_db_err};
use crate::store::repository::Repository;

/// The central database store.
///
/// `Clone` is cheap because the connection is stored behind an `Arc`.
///
/// # Construction
///
/// ```rust,ignore
/// let store = SeaOrmStore::connect_and_migrate(&cfg).await?;
/// ```
#[derive(Clone)]
pub struct SeaOrmStore {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmStore {
    /// Connect to Postgres and run pending migrations.
    pub async fn connect_and_migrate(cfg: &Config) -> OrmResult<Self> {
        let mut opts = ConnectOptions::new(&cfg.store.url);
        opts.max_connections(cfg.store.max_connections)
            .min_connections(cfg.store.min_connections)
            .connect_timeout(Duration::from_secs(cfg.store.connect_timeout_secs))
            .acquire_timeout(Duration::from_secs(cfg.store.acquire_timeout_secs))
            .idle_timeout(Duration::from_secs(cfg.store.idle_timeout_secs))
            .max_lifetime(Duration::from_secs(cfg.store.max_lifetime_secs))
            .sqlx_logging(cfg.store.sqlx_logging)
            .sqlx_logging_level(log::LevelFilter::Debug);

        info!("connecting to postgres…");
        let db = Database::connect(opts).await.map_err(map_db_err)?;

        info!("store ready");
        Ok(Self { db: Arc::new(db) })
    }

    pub fn db(&self) -> &DatabaseConnection {
        self.db.as_ref()
    }

    /// Build a typed repository for a generated SeaORM entity.
    pub fn repository<E>(&self) -> Repository<E>
    where
        E: sea_orm::EntityTrait,
        E::Model: Send + Sync,
    {
        Repository::from_shared(Arc::clone(&self.db))
    }

    /// Liveness / readiness check.
    pub async fn ping(&self) -> Result<(), DbErr> {
        self.db.as_ref().ping().await
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
        let tx = self.db.as_ref().begin().await.map_err(map_db_err)?;
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

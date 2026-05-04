//! Thin repository helpers for generated SeaORM entities.
//!
//! Repositories are cheap to construct (they hold an `Arc<DatabaseConnection>`
//! clone). Generated entities remain the source of truth for table shape,
//! columns, relations, and active models.
//!
//! This module is intentionally small. Put cross-entity CRUD helpers here, and
//! put domain-specific queries such as `find_by_email`, `find_by_slug`, or
//! soft-delete behavior in entity-specific repositories or service code.
//!
//! # Usage
//!
//! ```rust,ignore
//! use sea_orm::{ActiveValue::Set, ColumnTrait, QueryFilter};
//!
//! let repo = store.repository::<workspace::Entity>();
//!
//! let created = repo
//!     .insert(workspace::ActiveModel {
//!         name: Set("default".to_owned()),
//!         ..Default::default()
//!     })
//!     .await?;
//!
//! let workspace = repo.get_by_id(created.id).await?;
//! let page = repo.list(&Page::default(), |q| {
//!     q.filter(workspace::Column::OwnerId.eq(user_id))
//! }).await?;
//! ```

use std::{marker::PhantomData, sync::Arc};

use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, PaginatorTrait,
    PrimaryKeyTrait, Select,
};

use crate::error::{OrmError, OrmResult, map_db_err};
use crate::types::{Page, Paginated};

/// Generic typed repository for generated SeaORM entity `E`.
pub struct Repository<E: EntityTrait> {
    db: Arc<DatabaseConnection>,
    _entity: PhantomData<E>,
}

impl<E: EntityTrait> Clone for Repository<E> {
    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
            _entity: PhantomData,
        }
    }
}

impl<E> Repository<E>
where
    E: EntityTrait,
    E::Model: Send + Sync,
{
    /// Create a repository from an existing SeaORM database connection.
    ///
    /// The connection is stored behind an `Arc`, so repositories can be cloned
    /// and passed around without cloning the underlying connection enum.
    pub fn new(db: DatabaseConnection) -> Self {
        Self::from_shared(Arc::new(db))
    }

    /// Create a repository from a shared SeaORM database connection.
    pub fn from_shared(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db,
            _entity: PhantomData,
        }
    }

    /// Insert an active model and return the inserted model.
    ///
    /// The active model is still built by the generated entity module. This
    /// helper only centralizes connection handling and error mapping.
    pub async fn insert<A>(&self, active_model: A) -> OrmResult<E::Model>
    where
        A: ActiveModelTrait<Entity = E> + Send,
        E::Model: IntoActiveModel<A>,
    {
        E::insert(active_model)
            .exec_with_returning(self.db.as_ref())
            .await
            .map_err(map_db_err)
    }

    /// Update an active model and return the updated model.
    ///
    /// The caller must provide an active model with the primary key populated,
    /// as required by SeaORM's update API.
    pub async fn update<A>(&self, active_model: A) -> OrmResult<E::Model>
    where
        A: ActiveModelTrait<Entity = E> + Send,
        E::Model: IntoActiveModel<A>,
    {
        E::update(active_model)
            .exec(self.db.as_ref())
            .await
            .map_err(map_db_err)
    }

    /// Find a model by primary key.
    pub async fn find_by_id<T>(&self, id: T) -> OrmResult<Option<E::Model>>
    where
        T: Into<<E::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    {
        E::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(map_db_err)
    }

    /// Find a model by primary key, returning `OrmError::NotFound` if missing.
    pub async fn get_by_id<T>(&self, id: T) -> OrmResult<E::Model>
    where
        T: Into<<E::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    {
        self.find_by_id(id).await?.ok_or(OrmError::NotFound)
    }

    /// Paginated list.  `filter_fn` lets the caller inject arbitrary
    /// filters, ordering, joins, or selects without exposing raw SQL.
    pub async fn list<F>(&self, page: &Page, filter_fn: F) -> OrmResult<Paginated<E::Model>>
    where
        F: FnOnce(Select<E>) -> Select<E> + Send,
    {
        let q = filter_fn(E::find());
        let pag = q.paginate(self.db.as_ref(), page.per_page());
        let total = pag.num_items().await.map_err(map_db_err)?;
        let items = pag
            .fetch_page(page.zero_indexed())
            .await
            .map_err(map_db_err)?;

        Ok(Paginated::new(items, total, page))
    }

    /// Delete by primary key and return the number of affected rows.
    pub async fn delete_by_id<T>(&self, id: T) -> OrmResult<u64>
    where
        T: Into<<E::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    {
        let result = E::delete_by_id(id)
            .exec(self.db.as_ref())
            .await
            .map_err(map_db_err)?;
        Ok(result.rows_affected)
    }
}

// Allow entity repos to grab the raw connection for custom queries.
impl<E: EntityTrait> Repository<E> {
    /// Borrow the underlying database connection for custom SeaORM queries.
    pub fn db(&self) -> &DatabaseConnection {
        self.db.as_ref()
    }
}

/// Build a typed repository from a database connection.
///
/// This is a convenience wrapper around `Repository::new` for call sites that
/// prefer type inference:
///
/// ```rust,ignore
/// let db = sea_orm::Database::connect(database_url).await?;
/// let repo = repo::<workspace::Entity>(db);
/// ```
pub fn repo<R>(db: DatabaseConnection) -> Repository<R>
where
    R: EntityTrait,
    R::Model: Send + Sync,
{
    Repository::new(db)
}

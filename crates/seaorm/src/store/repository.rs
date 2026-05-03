//! Thin repository helpers for generated SeaORM entities.
//!
//! Repositories are cheap to construct (they hold a `DatabaseConnection`
//! clone). Generated entities remain the source of truth.
//!
//! # Usage
//!
//! ```rust,ignore
//! let repo = Repository::<workspace::Entity>::new(store.db().clone());
//!
//! let ws = repo.find_by_id(id).await?;
//! let page = repo.list(&Page::default(), |q| {
//!     q.filter(workspace::Column::OwnerId.eq(user_id))
//! }).await?;
//! ```

use std::marker::PhantomData;

use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, PrimaryKeyTrait, Select};

use crate::error::{OrmError, OrmResult, map_db_err};
use crate::types::{Page, Paginated};

/// Generic typed repository for generated SeaORM entity `E`.
pub struct Repository<E: EntityTrait> {
    db: DatabaseConnection,
    _entity: PhantomData<E>,
}

impl<E: EntityTrait> Clone for Repository<E> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            _entity: PhantomData,
        }
    }
}

impl<E> Repository<E>
where
    E: EntityTrait,
    E::Model: Send + Sync,
{
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            _entity: PhantomData,
        }
    }

    pub async fn find_by_id<T>(&self, id: T) -> OrmResult<Option<E::Model>>
    where
        T: Into<<E::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    {
        E::find_by_id(id).one(&self.db).await.map_err(map_db_err)
    }

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
        let pag = q.paginate(&self.db, page.per_page);
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
            .exec(&self.db)
            .await
            .map_err(map_db_err)?;
        Ok(result.rows_affected)
    }
}

// Allow entity repos to grab the raw connection for custom queries.
impl<E: EntityTrait> Repository<E> {
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

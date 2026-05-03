//! Generic `Repository<M>` — typed CRUD + soft-delete + pagination for any
//! entity that implements `SeaOrmModel`.
//!
//! Repositories are cheap to construct (they hold a `DatabaseConnection`
//! clone) and are the recommended way to interact with domain entities.
//!
//! # Usage
//!
//! ```rust,ignore
//! let repo = Repository::<workspace::Model>::new(store.db().clone());
//!
//! let ws = repo.find_by_id(id).await?;
//! let page = repo.list(&Page::default(), |q| q.filter(Column::OwnerId.eq(user_id))).await?;
//! repo.soft_delete(id).await?;
//! ```

use std::sync::Arc;

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction,
    EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use uuid::Uuid;

use crate::error::{OrmError, OrmResult, cancelled_by_hook, map_db_err};
use crate::hooks::{HookControl, HookCtx, OrmHook};
use crate::schema::SeaOrmModel;
use crate::types::{CreateFields, Page, Paginated, UpdateFields};

/// Generic typed repository for entity `M`.
pub struct Repository<M: SeaOrmModel> {
    db: DatabaseConnection,
    hooks: Vec<Arc<dyn OrmHook<M>>>,
}

impl<M: SeaOrmModel> Clone for Repository<M> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            hooks: self.hooks.clone(),
        }
    }
}

impl<M: SeaOrmModel> Repository<M> {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            hooks: Vec::new(),
        }
    }

    pub fn hook<H: OrmHook<M> + 'static>(mut self, hook: H) -> Self {
        self.hooks.push(Arc::new(hook));
        self
    }

    fn ctx<'a>(&'a self, tx: Option<&'a DatabaseTransaction>) -> HookCtx<'a> {
        HookCtx { db: &self.db, tx }
    }

    pub async fn find_by_id(&self, id: Uuid) -> OrmResult<Option<M>> {
        M::Entity::find()
            .filter(M::id_column().eq(id))
            .one(&self.db)
            .await
            .map_err(map_db_err)
    }

    pub async fn get_by_id(&self, id: Uuid) -> OrmResult<M> {
        self.find_by_id(id).await?.ok_or(OrmError::NotFound)
    }

    /// Paginated list.  `filter_fn` lets the caller inject arbitrary
    /// additional `filter` / `order_by` clauses without exposing raw SQL.
    pub async fn list<F>(&self, page: &Page, filter_fn: F) -> OrmResult<Paginated<M>>
    where
        F: Fn(sea_orm::Select<M::Entity>) -> sea_orm::Select<M::Entity> + Send,
    {
        let base = M::Entity::find().order_by_desc(M::created_at_column());

        let q = filter_fn(base);
        let pag = q.paginate(&self.db, page.per_page);
        let total = pag.num_items().await.map_err(map_db_err)?;
        let items = pag
            .fetch_page(page.zero_indexed())
            .await
            .map_err(map_db_err)?;

        Ok(Paginated::new(items, total, page))
    }

    /// Insert a new row.  `entity_fn` receives the active model produced by
    /// `SeaOrmModel::new_active` and may set entity-specific fields before
    /// the insert fires.
    pub async fn insert<F>(&self, mut create: CreateFields, entity_fn: F) -> OrmResult<M>
    where
        F: FnOnce(M::ActiveModel) -> M::ActiveModel + Send,
    {
        let ctx = self.ctx(None);
        for hook in &self.hooks {
            if hook.before_insert(&mut create, &ctx).await?.is_cancelled() {
                return Err(cancelled_by_hook("insert"));
            }
        }
        let now = Utc::now();
        let id = Uuid::new_v4();
        let active = entity_fn(M::new_active(id, create, now));
        let model = active.insert(&self.db).await.map_err(map_db_err)?;
        for hook in &self.hooks {
            hook.after_insert(&model, &ctx).await?;
        }
        Ok(model)
    }

    /// Same as [`insert`] but runs inside an existing transaction.
    pub async fn insert_in_tx<F>(
        &self,
        tx: &DatabaseTransaction,
        mut create: CreateFields,
        entity_fn: F,
    ) -> OrmResult<M>
    where
        F: FnOnce(M::ActiveModel) -> M::ActiveModel + Send,
    {
        let ctx = self.ctx(Some(tx));
        for hook in &self.hooks {
            if hook.before_insert(&mut create, &ctx).await?.is_cancelled() {
                return Err(cancelled_by_hook("insert"));
            }
        }
        let now = Utc::now();
        let active = entity_fn(M::new_active(Uuid::new_v4(), create, now));
        let model = active.insert(tx).await.map_err(map_db_err)?;
        for hook in &self.hooks {
            hook.after_insert(&model, &ctx).await?;
        }
        Ok(model)
    }

    /// Update plugin-level fields and stamp `updated_at`.
    /// The caller also supplies `entity_fn` to set entity-specific fields.
    pub async fn update<F>(&self, id: Uuid, mut update: UpdateFields, entity_fn: F) -> OrmResult<M>
    where
        F: FnOnce(M::ActiveModel) -> M::ActiveModel + Send,
    {
        let ctx = self.ctx(None);
        for hook in &self.hooks {
            if hook
                .before_update(&id, &mut update, &ctx)
                .await?
                .is_cancelled()
            {
                return Err(cancelled_by_hook("update"));
            }
        }
        let existing = self.get_by_id(id).await?;
        let mut active = existing.into_active_model();
        active = entity_fn(active);
        M::stamp_updated_at(&mut active, &update, Utc::now());
        let model = active.update(&self.db).await.map_err(map_db_err)?;
        for hook in &self.hooks {
            hook.after_update(&model, &ctx).await?;
        }
        Ok(model)
    }

    /// Hard delete by id.
    pub async fn delete(&self, id: Uuid) -> OrmResult<()> {
        let ctx = self.ctx(None);
        let model = self.get_by_id(id).await?;
        for hook in &self.hooks {
            if hook.before_delete(&model, &ctx).await?.is_cancelled() {
                return Err(cancelled_by_hook("delete"));
            }
        }
        M::Entity::delete_many()
            .filter(M::id_column().eq(id))
            .exec(&self.db)
            .await
            .map_err(map_db_err)?;
        for hook in &self.hooks {
            hook.after_delete(&model, &ctx).await?;
        }
        Ok(())
    }

    /// Soft-delete: set `deleted_at = now()`.
    /// No-op if the entity does not have a `deleted_at` field.
    pub async fn soft_delete(&self, id: Uuid) -> OrmResult<M> {
        let update = UpdateFields {
            deleted_at: Some(Utc::now()),
            ..Default::default()
        };
        self.update(id, update, |a| a).await
    }
}

// Allow entity repos to grab the raw connection for custom queries.
impl<M: SeaOrmModel> Repository<M> {
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

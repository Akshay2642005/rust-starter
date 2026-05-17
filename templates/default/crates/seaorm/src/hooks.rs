//! Optional lifecycle hook traits for service-layer operations.
//!
//! Every method has a default no-op impl; implementors override only what they
//! need. Service code decides where hooks are stored and when they run.
//!
//! # Example — audit log hook
//!
//! ```rust,ignore
//! struct AuditHook { logger: Arc<AuditLogger> }
//!
//! #[async_trait]
//! impl<M> OrmHook<M> for AuditHook
//! where
//!     M: Send + Sync,
//! {
//!     async fn after_insert(&self, model: &M, _ctx: &HookCtx<'_>) -> OrmResult<()> {
//!         self.logger.record("insert").await?;
//!         Ok(())
//!     }
//! }
//! ```

use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DatabaseTransaction};

use crate::error::OrmResult;
use crate::types::{CreateFields, UpdateFields};

/// Control flow returned by `before_*` hook methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookControl {
    Continue,
    Cancel,
}

impl HookControl {
    #[inline]
    pub fn is_cancelled(self) -> bool {
        matches!(self, Self::Cancel)
    }
}

/// Context passed to every hook invocation.
pub struct HookCtx<'a> {
    pub db: &'a DatabaseConnection,
    /// Present when the operation runs inside a transaction.
    pub tx: Option<&'a DatabaseTransaction>,
}

/// Generic lifecycle hook for service or repository operations.
///
/// `M` is typically a generated SeaORM model (e.g. `workspace::Model`).
#[async_trait]
pub trait OrmHook<M: Send + Sync>: Send + Sync {
    async fn before_insert(
        &self,
        create: &mut CreateFields,
        ctx: &HookCtx<'_>,
    ) -> OrmResult<HookControl> {
        let _ = (create, ctx);
        Ok(HookControl::Continue)
    }

    async fn after_insert(&self, model: &M, ctx: &HookCtx<'_>) -> OrmResult<()> {
        let _ = (model, ctx);
        Ok(())
    }

    async fn before_update(
        &self,
        id: &uuid::Uuid,
        update: &mut UpdateFields,
        ctx: &HookCtx<'_>,
    ) -> OrmResult<HookControl> {
        let _ = (id, update, ctx);
        Ok(HookControl::Continue)
    }

    async fn after_update(&self, model: &M, ctx: &HookCtx<'_>) -> OrmResult<()> {
        let _ = (model, ctx);
        Ok(())
    }

    async fn before_delete(&self, model: &M, ctx: &HookCtx<'_>) -> OrmResult<HookControl> {
        let _ = (model, ctx);
        Ok(HookControl::Continue)
    }

    async fn after_delete(&self, model: &M, ctx: &HookCtx<'_>) -> OrmResult<()> {
        let _ = (model, ctx);
        Ok(())
    }
}

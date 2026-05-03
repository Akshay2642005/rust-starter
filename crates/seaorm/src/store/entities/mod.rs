//! Entity module stubs.
//!
//! This directory is intentionally empty of concrete models.
//! Every application registers its own entities here.
//!
//! # How to add a domain entity
//!
//! 1. Create `src/store/entities/your_entity.rs`:
//!
//! ```rust,ignore
//! use crate::OrmEntity;
//! use sea_orm::entity::prelude::*;
//! use serde::Serialize;
//!
//! #[derive(OrmEntity, Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
//! #[sea_orm(table_name = "your_entities")]
//! pub struct Model {
//!     #[sea_orm(primary_key, auto_increment = false)]
//!     pub id: Uuid,
//!     // ... your fields ...
//!     pub created_at: DateTimeUtc,
//!     pub updated_at: DateTimeUtc,
//! }
//!
//! #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
//! pub enum Relation {}
//!
//! impl ActiveModelBehavior for ActiveModel {}
//! ```
//!
//! 2. Add a migration in `store/migrator.rs`.
//! 3. Build a typed repository (see `store/example/` for a full walkthrough).
//!
//! # Auth entities
//!
//! User / Session / Account / Verification are managed by `better-auth-rs`.
//! Do **not** define them here.

extern crate self as seaorm;

pub mod error;
pub mod hooks;
pub mod schema;
pub mod store;
pub mod types;

// Re-export the proc macro so consumers only need one dependency.
pub use seaorm_macros::OrmEntity;

// Re-export sea-orm so the derive macro can use `crate::sea_orm::*`.
pub use sea_orm;
pub use sea_orm::{Database, DatabaseConnection};

// Top-level convenience re-exports.
pub use error::{OrmError, OrmResult};
pub use hooks::{HookControl, HookCtx, OrmHook};
pub use schema::{AppSchema, OrmEntity, SeaOrmModel};
pub use store::SeaOrmStore;
pub use store::repository::Repository;

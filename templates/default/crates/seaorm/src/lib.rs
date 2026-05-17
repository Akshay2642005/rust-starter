extern crate self as seaorm;

pub mod error;
pub mod hooks;
pub mod store;
pub mod types;

// Re-export sea-orm so applications can use generated entities through one crate.
pub use sea_orm;
pub use sea_orm::{Database, DatabaseConnection};

// Top-level convenience re-exports.
pub use error::{OrmError, OrmResult};
pub use hooks::{HookControl, HookCtx, OrmHook};
pub use store::SeaOrmStore;
pub use store::repository::Repository;

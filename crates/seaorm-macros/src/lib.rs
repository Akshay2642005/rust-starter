//! Proc macros for `pg-seaorm`.

mod orm_entity;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Generates `OrmEntity` and `SeaOrmModel` trait impls for a SeaORM entity.
///
/// # Required fields
///
/// Every struct annotated with `OrmEntity` **must** declare:
/// - `id: Uuid`
/// - `created_at: DateTimeUtc`
/// - `updated_at: DateTimeUtc`
///
/// The macro refuses to compile if any of these are missing.
///
/// # Optional fields (opt-in by presence)
///
/// Declare the field on the struct and the macro wires it automatically:
/// - `deleted_at: Option<DateTimeUtc>` — soft-delete plugin
/// - `metadata: Json`                  — metadata plugin
/// - `slug: String`                    — slug plugin
///
/// Any other fields are treated as *entity-specific* and emitted as
/// `ActiveValue::NotSet` in `new_active`, letting database defaults apply.
///
/// # Usage
///
/// ```rust,ignore
/// #[derive(OrmEntity, Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
/// #[sea_orm(table_name = "workspaces")]
/// pub struct Model {
///     #[sea_orm(primary_key, auto_increment = false)]
///     pub id:          Uuid,
///     pub name:        String,
///     pub owner_id:    String,   // FK to better-auth user.id
///     pub slug:        String,   // slug plugin
///     pub metadata:    Json,     // metadata plugin
///     pub created_at:  DateTimeUtc,
///     pub updated_at:  DateTimeUtc,
///     pub deleted_at:  Option<DateTimeUtc>, // soft-delete plugin
/// }
/// ```
#[proc_macro_derive(OrmEntity)]
pub fn derive_orm_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    orm_entity::derive_orm_entity(&input).into()
}


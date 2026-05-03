//! Authoritative field registry for seaorm entity schemas.
//!
//! Shared by the `OrmEntity` proc macro (compile-time validation) and the
//! store layer (runtime dispatch).  **This is the single source of truth**
//! for which fields belong to a core schema vs which are plugin-provided.
//!
//! # Adding a new entity kind
//!
//! 1. Add a variant to [`EntityRole`].
//! 2. Add its core fields to the matching `*_CORE` static slice below.
//! 3. Add the arm to [`core_fields`] and [`core_field_names`].

#[derive(Clone, Copy, Debug)]
pub struct FieldDef {
    pub name: &'static str,
    pub ty: &'static str,
    pub is_primary_key: bool,
    pub column_name: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
pub struct AppSchema {
    pub name: &'static str,
    pub fields: &'static [FieldDef],
}

macro_rules! f {
    ($name:expr, $ty:expr) => {
        FieldDef {
            name: $name,
            ty: $ty,
            is_primary_key: false,
            column_name: None,
        }
    };
}

macro_rules! pk {
    ($name:expr, $ty:expr) => {
        FieldDef {
            name: $name,
            ty: $ty,
            is_primary_key: true,
            column_name: None,
        }
    };
}

macro_rules! f_col {
    ($name:expr, $ty:expr, $col:expr) => {
        FieldDef {
            name: $name,
            ty: $ty,
            is_primary_key: false,
            column_name: Some($col),
        }
    };
}

pub static ENTITY_REQUIRED_FIELDS: &[FieldDef] = &[
    pk!("id", "Uuid"),
    f!("created_at", "DateTimeUtc"),
    f!("updated_at", "DateTimeUtc"),
];

pub static APPSCHEMA: &[AppSchema] = &[
    AppSchema {
        name: "soft-delete",
        fields: &[f!("deleted_at", "Option<DateTimeUtc>")],
    },
    AppSchema {
        name: "metadata",
        fields: &[f_col!("metadata", "Json", "metadata")],
    },
    AppSchema {
        name: "slug",
        fields: &[f!("slug", "String")],
    },
];

pub fn required_field_names() -> Vec<&'static str> {
    ENTITY_REQUIRED_FIELDS.iter().map(|f| f.name).collect()
}

pub fn all_schema_field_names() -> Vec<&'static str> {
    APPSCHEMA
        .iter()
        .flat_map(|p| p.fields.iter())
        .map(|f| f.name)
        .collect()
}

pub fn is_schema_field(name: &str) -> bool {
    all_schema_field_names().contains(&name)
}

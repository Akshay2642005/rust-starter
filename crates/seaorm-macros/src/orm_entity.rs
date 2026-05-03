//! Implementation of `#[derive(OrmEntity)]`.
//!
//! The macro does three things:
//!   1. Validates that all required fields (id, created_at, updated_at) are present.
//!   2. Generates a `SeaOrmModel` trait impl with `new_active` / `apply_update`.
//!   3. Generates a domain-level `OrmEntity` trait impl for field accessors.
//!
//! Plugin fields (soft-delete, metadata, slug) are detected automatically by
//! name — no extra attribute required.  Unknown extra fields are emitted as
//! `ActiveValue::NotSet` in `new_active` so database defaults apply.

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use seaorm_registry as registry;
use syn::{Data, DeriveInput, Fields};

// ── Crate path resolution ─────────────────────────────────────────────────────

fn resolve_root() -> TokenStream {
    match crate_name("seaorm") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(n)) => {
            let ident = Ident::new(&n, Span::call_site());
            quote!(::#ident)
        }
        Err(_) => syn::Error::new(
            Span::call_site(),
            "`seaorm` must be a dependency to use `#[derive(OrmEntity)]`",
        )
        .to_compile_error(),
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub(crate) fn derive_orm_entity(input: &DeriveInput) -> TokenStream {
    let root = resolve_root();

    // Collect field idents.
    let named_fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => f,
            _ => {
                return syn::Error::new_spanned(
                    &input.ident,
                    "OrmEntity requires a struct with named fields",
                )
                .to_compile_error();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input.ident, "OrmEntity requires a struct")
                .to_compile_error();
        }
    };

    let idents: Vec<Ident> = named_fields
        .named
        .iter()
        .filter_map(|f| f.ident.clone())
        .collect();

    // 1. Validate required fields.
    let required = registry::required_field_names();
    for req in &required {
        if !idents.iter().any(|i| i == req) {
            return syn::Error::new_spanned(
                &input.ident,
                format!("OrmEntity: missing required field `{req}`"),
            )
            .to_compile_error();
        }
    }

    let has = |name: &str| -> bool { idents.iter().any(|i| i == name) };

    // 2. Classify fields.
    // Extra = not in required and not a known plugin field.
    // These get `NotSet` in new_active so DB defaults fire.
    let all_known: Vec<&str> = required
        .iter()
        .chain(registry::all_schema_field_names().iter())
        .copied()
        .collect();

    let extra_not_set: Vec<TokenStream> = idents
        .iter()
        .filter(|i| !all_known.iter().any(|k| i == k))
        .map(|f| quote! { #f: #root::sea_orm::ActiveValue::NotSet })
        .collect();

    // 3. Plugin field token streams.
    let soft_delete_new = if has("deleted_at") {
        quote! { deleted_at: #root::sea_orm::ActiveValue::Set(None), }
    } else {
        quote!()
    };

    let metadata_new = if has("metadata") {
        quote! { metadata: #root::sea_orm::ActiveValue::Set(
            create.metadata.unwrap_or_else(|| ::serde_json::json!({}))
        ), }
    } else {
        quote!()
    };

    let slug_new = if has("slug") {
        quote! { slug: #root::sea_orm::ActiveValue::Set(
            create.slug.expect("slug is required when the slug plugin is enabled")
        ), }
    } else {
        quote!()
    };

    let soft_delete_apply = if has("deleted_at") {
        quote! {
            if let Some(v) = update.deleted_at {
                active.deleted_at = #root::sea_orm::ActiveValue::Set(Some(v));
            }
        }
    } else {
        quote!()
    };

    let metadata_apply = if has("metadata") {
        quote! {
            if let Some(v) = update.metadata { active.metadata = #root::sea_orm::ActiveValue::Set(v); }
        }
    } else {
        quote!()
    };

    let slug_apply = if has("slug") {
        quote! {
            if let Some(v) = update.slug { active.slug = #root::sea_orm::ActiveValue::Set(v); }
        }
    } else {
        quote!()
    };

    // 4. OrmEntity accessor impls.
    let deleted_at_accessor = if has("deleted_at") {
        quote! { fn deleted_at(&self) -> Option<::chrono::DateTime<::chrono::Utc>> { self.deleted_at } }
    } else {
        quote! { fn deleted_at(&self) -> Option<::chrono::DateTime<::chrono::Utc>> { None } }
    };
    let metadata_accessor = if has("metadata") {
        quote! { fn metadata(&self) -> Option<&::serde_json::Value> { Some(&self.metadata) } }
    } else {
        quote! { fn metadata(&self) -> Option<&::serde_json::Value> { None } }
    };
    let slug_accessor = if has("slug") {
        quote! { fn slug(&self) -> Option<&str> { Some(&self.slug) } }
    } else {
        quote! { fn slug(&self) -> Option<&str> { None } }
    };

    let ident = &input.ident;

    quote! {
        // ── OrmEntity: domain-level field accessors ───────────────────────────
        impl #root::OrmEntity for #ident {
            fn id(&self)         -> ::uuid::Uuid               { self.id }
            fn created_at(&self) -> ::chrono::DateTime<::chrono::Utc> { self.created_at }
            fn updated_at(&self) -> ::chrono::DateTime<::chrono::Utc> { self.updated_at }
            #deleted_at_accessor
            #metadata_accessor
            #slug_accessor
        }

        // ── SeaOrmModel: new_active / apply_update ────────────────────────────
        impl #root::SeaOrmModel for #ident {
            type Entity      = Entity;
            type ActiveModel = ActiveModel;
            type Column      = Column;

            fn id_column()         -> Self::Column { Column::Id }
            fn created_at_column() -> Self::Column { Column::CreatedAt }
            fn updated_at_column() -> Self::Column { Column::UpdatedAt }

            fn new_active(
                id:     ::uuid::Uuid,
                create: #root::types::CreateFields,
                now:    ::chrono::DateTime<::chrono::Utc>,
            ) -> Self::ActiveModel {
                Self::ActiveModel {
                    id:         #root::sea_orm::ActiveValue::Set(id),
                    created_at: #root::sea_orm::ActiveValue::Set(now),
                    updated_at: #root::sea_orm::ActiveValue::Set(now),
                    #soft_delete_new
                    #metadata_new
                    #slug_new
                    #(#extra_not_set,)*
                }
            }

            fn stamp_updated_at(
                active: &mut Self::ActiveModel,
                update: &#root::types::UpdateFields,
                now:    ::chrono::DateTime<::chrono::Utc>,
            ) {
                #soft_delete_apply
                #metadata_apply
                #slug_apply
                active.updated_at = #root::sea_orm::ActiveValue::Set(now);
            }
        }
    }
}

//! Route macro expansion and registration helpers.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Ident, ItemFn, LitStr, Result, Token, parse::Parse, parse::ParseStream, parse_macro_input,
};

/// Parsed arguments for the `#[route(METHOD, "/path")]` attribute.
struct RouteArgs {
    method: Ident,
    _comma: Token![,],
    path: LitStr,
}

impl Parse for RouteArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            method: input.parse()?,
            _comma: input.parse()?,
            path: input.parse()?,
        })
    }
}

/// Expands the `#[route]` attribute into a registered Axum route.
pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as RouteArgs);
    let item_fn = parse_macro_input!(item as ItemFn);

    match expand_inner(args, item_fn) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// Builds the route registration module and inventory submission.
fn expand_inner(args: RouteArgs, item_fn: ItemFn) -> Result<proc_macro2::TokenStream> {
    let method = args.method.to_string().to_uppercase();
    let route_fn = match method.as_str() {
        "GET" => quote!(axum::routing::get),
        "POST" => quote!(axum::routing::post),
        "PUT" => quote!(axum::routing::put),
        "PATCH" => quote!(axum::routing::patch),
        "DELETE" => quote!(axum::routing::delete),
        _ => {
            return Err(syn::Error::new(
                args.method.span(),
                "route method must be one of GET, POST, PUT, PATCH, DELETE",
            ));
        }
    };

    let path = args.path;
    let ident = &item_fn.sig.ident;
    let register_mod = format_ident!("__route_registration_{}", ident);
    let register_fn = format_ident!("__register_{}", ident);

    Ok(quote! {
        #item_fn

        #[allow(non_snake_case)]
        mod #register_mod {
            use super::*;

            fn #register_fn(
                router: axum::Router<crate::state::AppState>,
            ) -> axum::Router<crate::state::AppState> {
                let path = crate::registry::scoped_path(#path);
                router.route(&path, #route_fn(super::#ident))
            }

            inventory::submit! {
                crate::registry::RouteEntry::new(#register_fn)
            }
        }
    })
}

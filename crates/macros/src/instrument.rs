//! Handler instrumentation macro expansion.

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Expands `#[instrument_handler]` into a tracing-instrumented async handler.
pub fn expand(item: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(item as ItemFn);

    if item_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            item_fn.sig.fn_token,
            "#[instrument_handler] can only be used on async functions",
        )
        .to_compile_error()
        .into();
    }

    let attrs = &item_fn.attrs;
    let vis = &item_fn.vis;
    let sig = &item_fn.sig;
    let block = &item_fn.block;
    let ident = &item_fn.sig.ident;

    quote! {
        #(#attrs)*
        #[tracing::instrument(
            skip_all,
            fields(
                handler = stringify!(#ident),
                request_id = tracing::field::Empty,
                user_id = tracing::field::Empty,
                status = tracing::field::Empty
            )
        )]
        #vis #sig {
            (async move #block).await
        }
    }
    .into()
}

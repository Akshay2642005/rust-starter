//! Graceful shutdown macro expansion.

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Expands `#[graceful_shutdown]` into a cross-platform shutdown handler.
pub fn expand(item: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(item as ItemFn);

    if item_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            item_fn.sig.fn_token,
            "#[graceful_shutdown] can only be used on async functions",
        )
        .to_compile_error()
        .into();
    }

    let attrs = &item_fn.attrs;
    let vis = &item_fn.vis;
    let sig = &item_fn.sig;

    quote! {
        #(#attrs)*
        #vis #sig {
            #[cfg(unix)]
            {
                use tokio::signal::unix::{signal, SignalKind};

                let mut terminate = match signal(SignalKind::terminate()) {
                    Ok(signal) => signal,
                    Err(error) => {
                        tracing::error!(error = %error, "failed to listen for SIGTERM");
                        if let Err(error) = tokio::signal::ctrl_c().await {
                            tracing::error!(error = %error, "failed to listen for shutdown signal");
                        }
                        return;
                    }
                };

                let mut interrupt = match signal(SignalKind::interrupt()) {
                    Ok(signal) => signal,
                    Err(error) => {
                        tracing::error!(error = %error, "failed to listen for SIGINT");
                        if let Err(error) = tokio::signal::ctrl_c().await {
                            tracing::error!(error = %error, "failed to listen for shutdown signal");
                        }
                        return;
                    }
                };

                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {},
                    _ = terminate.recv() => {},
                    _ = interrupt.recv() => {},
                }
            }

            #[cfg(not(unix))]
            {
                if let Err(error) = tokio::signal::ctrl_c().await {
                    tracing::error!(error = %error, "failed to listen for shutdown signal");
                }
            }
        }
    }
    .into()
}

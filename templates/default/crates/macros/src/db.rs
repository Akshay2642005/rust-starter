//! Database instrumentation macro expansion.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Ident, ItemFn, LitStr, Result, Token, parse::Parse, parse::ParseStream, parse_macro_input,
};

/// Parsed arguments for the `#[instrument_db]` attribute.
struct DbArgs {
    statement: LitStr,
    db: Option<LitStr>,
}

impl Parse for DbArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut statement: Option<LitStr> = None;
        let mut db: Option<LitStr> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _eq: Token![=] = input.parse()?;
            let value: LitStr = input.parse()?;

            match key.to_string().as_str() {
                "statement" => statement = Some(value),
                "db" => db = Some(value),
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        "unsupported argument; expected `statement` or `db`",
                    ));
                }
            }

            if input.is_empty() {
                break;
            }

            let _comma: Token![,] = input.parse()?;
        }

        let Some(statement) = statement else {
            return Err(syn::Error::new(
                input.span(),
                "`statement` is required for #[instrument_db]",
            ));
        };

        Ok(Self { statement, db })
    }
}

/// Expands `#[instrument_db]` into a tracing-instrumented function.
pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as DbArgs);
    let item_fn = parse_macro_input!(item as ItemFn);

    let attrs = &item_fn.attrs;
    let vis = &item_fn.vis;
    let sig = &item_fn.sig;
    let block = &item_fn.block;

    let statement = args.statement;
    let db_field = args.db.map(|db| quote!(db.system = #db,));

    let fields = if let Some(db_field) = db_field {
        quote!(#db_field db.statement = #statement)
    } else {
        quote!(db.statement = #statement)
    };

    quote! {
        #(#attrs)*
        #[tracing::instrument(skip_all, fields(#fields))]
        #vis #sig {
            #block
        }
    }
    .into()
}

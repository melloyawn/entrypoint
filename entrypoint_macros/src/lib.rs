#![forbid(unsafe_code)]
#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![warn(clippy::all, clippy::cargo, clippy::nursery, clippy::pedantic)]
#![warn(clippy::unwrap_used)]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, DeriveInput, FnArg, Ident, ItemFn, Pat, PatIdent, PatType,
    Path, Type, TypePath,
};

#[proc_macro_derive(DotEnvDefault)]
pub fn derive_dotenv_parser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let output = quote! {
      impl entrypoint::DotEnvParser for #name {}
    };

    TokenStream::from(output)
}

#[proc_macro_derive(LoggerDefault, attributes(log_level))]
pub fn derive_logger(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let mut log_level: syn::PatPath = parse_quote! { entrypoint::tracing::Level::INFO };

    for attr in input.attrs {
        if attr.path().is_ident("log_level") {
            log_level = attr.parse_args().expect("invalid log_level provided");
        }
    }

    let output = quote! {
      impl entrypoint::Logger for #name {
          fn log_level(&self) -> entrypoint::tracing::Level {
              entrypoint::tracing::Level::into(#log_level)
          }
      }
    };

    TokenStream::from(output)
}

#[proc_macro_attribute]
pub fn entrypoint(_args: TokenStream, item: TokenStream) -> TokenStream {
    let tokens = parse_macro_input!(item as ItemFn);

    let attrs = { tokens.attrs };

    // you think there'd be a cleaner/easier way to do this...
    let (input_param_ident, input_param_type) = {
        let mut input_param_ident: Option<Ident> = None;
        let mut input_param_type: Option<Path> = None;

        let inputs = tokens.sig.inputs.clone();

        for input in inputs {
            if let FnArg::Typed(PatType {
                pat: name,
                ty: r#type,
                ..
            }) = input
            {
                match (*name, *r#type) {
                    // 2nd match to get boxed values
                    (
                        Pat::Ident(PatIdent { ident: name, .. }),
                        Type::Path(TypePath { path: r#type, .. }),
                    ) => {
                        input_param_ident = Some(name);
                        input_param_type = Some(r#type.clone());
                    }
                    (_, _) => {
                        continue;
                    }
                }
            }
        }

        (
            input_param_ident.expect("Could not determine input parameter name"),
            input_param_type.expect("Could not determine input parameter type"),
        )
    };

    let signature = {
        let mut signature = tokens.sig.clone();
        signature.ident = format_ident!("main");
        signature.inputs.clear();
        signature.output = parse_quote! {-> entrypoint::anyhow::Result<()>};
        signature
    };

    let block = { tokens.block };

    quote! {
      #(#attrs)*
      #signature {
        <#input_param_type as entrypoint::clap::Parser>::parse().entrypoint(|#input_param_ident| { #block })
      }
    }
    .into()
}

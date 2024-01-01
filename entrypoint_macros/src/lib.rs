//! macro(s) to improve [`entrypoint`] ergonomics
//!
//! This crate should not be imported directly, but rather accessed through the `macros` feature of [`entrypoint`].
//!
//! # Examples
//! ```
//! use entrypoint::prelude::*;
//!
//! #[derive(clap::Parser, DotEnvDefault, LoggerDefault, Debug)]
//! struct Args {
//!     #[arg(long, env)]
//!     verbose: bool,
//! }
//!
//! // this function replaces `main`
//! #[entrypoint::entrypoint]
//! fn entrypoint(args: Args) -> entrypoint::anyhow::Result<()> {
//!     info!("entrypoint input args: {:#?}", args);
//!     Ok(())
//! }
//! ```
//! [`entrypoint`]: https://docs.rs/entrypoint

#![no_std]
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

/// derive default impl(s) for [`entrypoint::DotEnvParser`]
///
/// # Examples
/// ```
/// # use entrypoint::prelude::*;
/// #[derive(clap::Parser, DotEnvDefault)]
/// struct Args {}
///
/// assert_eq!(Args::parse().additional_dotenv_files(), None);
/// ```
/// [`entrypoint::DotEnvParser`]: https://docs.rs/entrypoint/latest/entrypoint/trait.DotEnvParser.html
#[proc_macro_derive(DotEnvDefault)]
pub fn derive_dotenv_parser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let output = quote! {
      impl entrypoint::DotEnvParser for #name {}
    };

    TokenStream::from(output)
}

/// derive default impl(s) for [`entrypoint::Logger`]
///
/// # Attributes
/// * `#[log_level]` sets the default [`tracing_subscriber` verbosity level].
///
/// # Panics
/// * `#[log_level]` has missing or malformed input
///
/// # Examples
/// ```
/// # use entrypoint::prelude::*;
/// #[derive(clap::Parser, LoggerDefault)]
/// #[log_level(entrypoint::tracing_subscriber::filter::LevelFilter::DEBUG)]
/// struct Args {}
///
/// assert_eq!(Args::parse().log_level(), entrypoint::tracing_subscriber::filter::LevelFilter::DEBUG);
/// ```
/// [`entrypoint::Logger`]: https://docs.rs/entrypoint/latest/entrypoint/trait.Logger.html
/// [`tracing_subscriber` verbosity level]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.LevelFilter.html
#[proc_macro_derive(LoggerDefault, attributes(log_level))]
pub fn derive_logger(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let mut log_level: syn::PatPath =
        parse_quote! { tracing_subscriber::fmt::Subscriber::DEFAULT_MAX_LEVEL };

    for attr in input.attrs {
        if attr.path().is_ident("log_level") {
            log_level = attr
                .parse_args()
                .expect("required log_level input parameter is missing or malformed");
        }
    }

    let output = quote! {
      impl entrypoint::Logger for #name {
          fn log_level(&self) -> entrypoint::tracing_subscriber::filter::LevelFilter {
              #log_level
          }
      }
    };

    TokenStream::from(output)
}

/// marks function as [`entrypoint`] `function` (i.e. the `main` replacement)
///
/// **Ordering may matter when used with other attribute macros.**
///
/// # Panics
/// * candidate function has missing or malformed input parameter
///
/// # Examples
/// ```
/// # use entrypoint::prelude::*;
/// #[derive(clap::Parser, DotEnvDefault, LoggerDefault, Debug)]
/// struct Args {}
///
/// // this function replaces `main`
/// #[entrypoint::entrypoint]
/// fn entrypoint(args: Args) -> entrypoint::anyhow::Result<()> {
///     info!("this is my main function!");
///     # let args = args;
///     Ok(())
/// }
/// ```
/// [`entrypoint`]: https://docs.rs/entrypoint/latest/entrypoint/trait.Entrypoint.html#method.entrypoint
#[proc_macro_attribute]
pub fn entrypoint(_args: TokenStream, item: TokenStream) -> TokenStream {
    let tokens = parse_macro_input!(item as ItemFn);

    let attrs = { tokens.attrs };

    // you think there'd be a cleaner/easier way to do this...
    let (input_param_ident, input_param_type) = {
        let mut input_param_ident: Option<Ident> = None;
        let mut input_param_type: Option<Path> = None;

        for input in tokens.sig.inputs.clone() {
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
            input_param_ident.expect("required entrypoint input parameter is missing or malformed"),
            input_param_type.expect("required entrypoint input parameter is missing or malformed"),
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

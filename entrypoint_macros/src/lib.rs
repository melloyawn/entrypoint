//! macro(s) to improve [`entrypoint`] ergonomics
//!
//! This crate should not be imported directly, but rather accessed through the `macros` feature of [`entrypoint`].
//!
//! # Examples
//! ```
//! use entrypoint::prelude::*;
//!
//! #[derive(clap::Parser, DotEnvDefault, LoggerDefault, Debug)]
//! struct Args {}
//!
//! // this function replaces `main()`
//! #[entrypoint::entrypoint]
//! fn main(args: Args) -> entrypoint::anyhow::Result<()> {
//!     info!("entrypoint input args: {:#?}", args);
//!     Ok(())
//! }
//! ```
//! [`entrypoint`]: https://docs.rs/entrypoint

#![no_std]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, DeriveInput, FnArg, Ident, ItemFn, Pat, PatIdent, PatType,
    Path, Type, TypePath,
};

/// derive default impl(s) for [`entrypoint::DotEnvParserConfig`]
///
/// # Examples
/// ```
/// # use entrypoint::prelude::*;
/// #[derive(clap::Parser, DotEnvDefault)]
/// struct Args {}
///
/// // uses default implementation(s)
/// assert_eq!(Args::parse().additional_dotenv_files(), None);
/// ```
/// [`entrypoint::DotEnvParserConfig`]: https://docs.rs/entrypoint/latest/entrypoint/trait.DotEnvParserConfig.html
#[proc_macro_derive(DotEnvDefault)]
pub fn derive_dotenv_parser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let output = quote! {
      impl entrypoint::DotEnvParserConfig for #name {}
    };

    TokenStream::from(output)
}

/// derive default impl(s) for [`entrypoint::LoggerConfig`]
///
/// # Attributes
/// * `#[log_format]` sets the default [`tracing_subscriber::Format`]. Defaults to `default`. Valid options are:
///   * [`compact`]
///   * [`default`]
///   * [`full`]
///   * [`json`]
///   * [`pretty`]
/// * `#[log_level]`  sets the default [`tracing_subscriber::LevelFilter`]. Defaults to [`DEFAULT_MAX_LEVEL`].
/// * `#[log_writer]` sets the default [`tracing_subscriber::MakeWriter`]. Defaults to [`std::io::stdout`].
///
/// # Panics
/// * `#[log_format]` has missing or malformed input
/// * `#[log_level]`  has missing or malformed input
/// * `#[log_writer]` has missing or malformed input
///
/// # Examples
/// ```
/// # use entrypoint::prelude::*;
/// #[derive(clap::Parser, LoggerDefault)]
/// #[log_format(json)]
/// #[log_level(entrypoint::tracing_subscriber::filter::LevelFilter::DEBUG)]
/// #[log_writer(std::io::stderr)]
/// struct Args {}
///
/// # //#FIXME - test format #
/// # //#FIXME - test writer #
/// # //#FIXME - test level  # assert!(enabled!(entrypoint::Level::DEBUG));
/// ```
/// [`compact`]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/format/struct.Compact.html
/// [`default`]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/format/struct.Format.html#method.default
/// [`full`]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/format/struct.Full.html
/// [`json`]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/format/struct.Json.html
/// [`pretty`]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/format/struct.Pretty.html
/// [`DEFAULT_MAX_LEVEL`]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/struct.Subscriber.html#associatedconstant.DEFAULT_MAX_LEVEL
/// [`std::io::stdout`]: https://doc.rust-lang.org/std/io/fn.stdout.html
/// [`entrypoint::LoggerConfig`]: https://docs.rs/entrypoint/latest/entrypoint/trait.LoggerConfig.html
/// [`tracing_subscriber::Format`]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/format/struct.Format.html
/// [`tracing_subscriber::LevelFilter`]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.LevelFilter.html
/// [`tracing_subscriber::MakeWriter`]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/writer/trait.MakeWriter.html
#[proc_macro_derive(LoggerDefault, attributes(log_format, log_level, log_writer))]
pub fn derive_logger(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let mut log_format: syn::ExprCall = parse_quote! { clone() };
    let mut log_level: syn::ExprPath =
        parse_quote! { tracing_subscriber::fmt::Subscriber::DEFAULT_MAX_LEVEL };
    let mut log_writer: syn::ExprPath = parse_quote! { std::io::stdout };

    for attr in input.attrs {
        if attr.path().is_ident("log_format") {
            let key: syn::ExprPath = attr
                .parse_args()
                .expect("required log_format input parameter is missing or malformed");
            log_format = if key.path.is_ident("compact") {
                parse_quote! { compact() }
            } else if key.path.is_ident("default") || key.path.is_ident("full") {
                parse_quote! { clone() }
            } else if key.path.is_ident("json") {
                parse_quote! { json() }
            } else if key.path.is_ident("pretty") {
                parse_quote! { pretty() }
            } else {
                panic!(
                    "log_format input parameter is unknown type: {:?}",
                    key.path.get_ident()
                );
            };
        } else if attr.path().is_ident("log_level") {
            log_level = attr
                .parse_args()
                .expect("required log_level input parameter is missing or malformed");
        } else if attr.path().is_ident("log_writer") {
            log_writer = attr
                .parse_args()
                .expect("required log_writer input parameter is missing or malformed");
        }
    }

    let output = quote! {
      impl entrypoint::LoggerConfig for #name {
          fn default_log_format<S, N>(&self) -> impl FormatEvent<S, N> + Send + Sync + 'static
          where
              S: Subscriber + for<'a> LookupSpan<'a>,
              N: for<'writer> FormatFields<'writer> + 'static,
          {
              Format::default().#log_format
          }

          fn default_log_level(&self) -> entrypoint::tracing_subscriber::filter::LevelFilter {
              #log_level
          }

          fn default_log_writer(&self) -> impl for<'writer> MakeWriter<'writer> + Send + Sync + 'static {
              #log_writer
          }
      }
    };

    TokenStream::from(output)
}

/// marks function as [`entrypoint`] `function` (i.e. the `main()` replacement)
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
/// fn main(args: Args) -> entrypoint::anyhow::Result<()> {
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

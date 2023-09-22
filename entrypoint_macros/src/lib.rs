#![forbid(unsafe_code)]
#![deny(unreachable_pub, private_in_public)]
#![warn(
    clippy::all,
    clippy::dbg_macro,
    clippy::unused_self,
    clippy::macro_use_imports,
    missing_docs
)]

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn entrypoint(_args: TokenStream, item: TokenStream) -> TokenStream {
    // Args::parse().entrypoint(run_app)
    item
}




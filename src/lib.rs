//! #FIXME

#![forbid(unsafe_code)]

pub use anyhow;
pub use clap;
pub use tracing;

use anyhow::Result;
use clap::Parser;
use tracing::info;

/// #FIXME
pub trait Entrypoint: Parser {
    fn additional_startup_config(self) -> Result<Self> {
        Ok(self)
    }

    /// #FIXME
    fn entrypoint<F>(self, function: F) -> Result<()>
    where
        F: FnOnce(Self) -> Result<()>,
    {
        let entrypoint = { self.additional_startup_config()? };

        info!("executing entrypoint() function");
        function(entrypoint)
    }
}
impl<P: Parser> Entrypoint for P {}

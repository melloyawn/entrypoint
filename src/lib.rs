//! #FIXME

#![forbid(unsafe_code)]

pub use anyhow;
pub use clap;
pub use tracing;

use anyhow::Result;
use clap::Parser;
use tracing::info;

/// #FIXME
pub trait Entrypoint: Parser + LoggingConfig {
    /// #FIXME
    fn additional_startup_config(self) -> Result<Self> {
        Ok(self)
    }

    /// #FIXME
    fn entrypoint<F>(self, function: F) -> Result<()>
    where
        F: FnOnce(Self) -> Result<()>,
    {
        let entrypoint = {
            // use local/default logger until logging_config() sets global logger
            let _log = tracing::subscriber::set_default(tracing_subscriber::fmt().finish());

            self.logging_config()?.additional_startup_config()?
        };
        info!("setup/config complete; executing entrypoint");
        function(entrypoint)
    }
}
impl<P: Parser> Entrypoint for P {}

/// #FIXME
pub trait LoggingConfig: Parser {
    /// #FIXME
    fn logging_config(self) -> Result<Self> {
        let format = tracing_subscriber::fmt::format();

        tracing_subscriber::fmt().event_format(format).init();

        Ok(self)
    }
}
impl<P: Parser> LoggingConfig for P {}

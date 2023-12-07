#![forbid(unsafe_code)]
#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![warn(clippy::all, clippy::cargo, clippy::nursery, clippy::pedantic)]
#![warn(clippy::unwrap_used)]

////////////////////////////////////////////////////////////////////////////////
pub extern crate anyhow;
pub extern crate clap;
pub extern crate tracing;

////////////////////////////////////////////////////////////////////////////////
#[cfg(feature = "macros")]
pub extern crate entrypoint_macros;

#[cfg(feature = "macros")]
pub mod macros {
    pub use crate::entrypoint_macros::entrypoint;
}

////////////////////////////////////////////////////////////////////////////////
pub mod prelude {
    pub use crate::anyhow;
    pub use crate::anyhow::Result;

    pub use crate::clap;
    pub use crate::clap::Parser;

    pub use crate::tracing;
    pub use crate::tracing::{debug, error, info, trace, warn};

    pub use crate::DotEnvConfig;
    pub use crate::Entrypoint;
    pub use crate::LoggingConfig;

    #[cfg(feature = "macros")]
    pub use crate::macros::*;
}

pub use crate::prelude::*;

////////////////////////////////////////////////////////////////////////////////
pub trait Entrypoint: Parser + DotEnvConfig + LoggingConfig {
    fn additional_configuration(self) -> Result<Self> {
        Ok(self)
    }

    fn entrypoint<F, T>(self, function: F) -> Result<T>
    where
        F: FnOnce(Self) -> Result<T>,
    {
        let entrypoint = {
            // use local/default logger until configure_logging() sets global logger
            let _log = tracing::subscriber::set_default(tracing_subscriber::fmt().finish());

            self.process_env_files()?
                .configure_logging()?
                .additional_configuration()?
                .dump_env_vars()
        };
        info!("setup/config complete; executing entrypoint");
        function(entrypoint)
    }
}
impl<P: Parser + DotEnvConfig + LoggingConfig> Entrypoint for P {}

pub trait LoggingConfig: Parser {
    fn configure_logging(self) -> Result<Self> {
        let format = tracing_subscriber::fmt::format();

        // #FIXME use try_init() instead?
        tracing_subscriber::fmt().event_format(format).init();

        Ok(self)
    }
}
impl<P: Parser> LoggingConfig for P {}

pub trait DotEnvConfig: Parser {
    /// user should/could override this
    /// order matters
    fn env_files(&self) -> Option<Vec<std::path::PathBuf>> {
        info!("env_files() default impl returns None");
        None
    }

    /// #FIXME - doc
    /// warning: debug level can leak secrets
    fn dump_env_vars(self) -> Self {
        for (key, value) in std::env::vars() {
            debug!("{key}: {value}");
        }

        self
    }

    /// order matters - env, .env, passed paths
    /// don't override this
    fn process_env_files(self) -> Result<Self> {
        // do twice in case `env_files()` is dependant on `.env` supplied variable
        for _ in 0..=1 {
            let processed_found_dotenv = dotenvy::dotenv().map_or(Err(()), |file| {
                info!("dotenv::from_filename({})", file.display());
                Ok(())
            });

            // #FIXME - use map_or() here too?
            let processed_supplied_dotenv = if let Some(files) = self.env_files() {
                for file in files {
                    info!("dotenv::from_filename({})", file.display());
                    dotenvy::from_filename(file)?;
                }
                Ok(())
            } else {
                Err(())
            };

            if processed_found_dotenv.is_err() && processed_supplied_dotenv.is_err() {
                info!("no dotenv file(s) found/processed");
                break;
            }
        }

        Ok(self)
    }
}

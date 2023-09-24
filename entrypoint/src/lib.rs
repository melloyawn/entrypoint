#![forbid(unsafe_code)]
#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![warn(clippy::all, clippy::cargo, clippy::nursery, clippy::pedantic)]
#![warn(clippy::unwrap_used)]

pub extern crate anyhow;
pub extern crate clap;
pub extern crate tracing;

pub mod prelude {
    pub use super::Entrypoint;

    pub use super::anyhow;
    pub use super::anyhow::Result;

    pub use super::clap;
    pub use super::clap::Parser;

    pub use super::tracing;
}

use crate::prelude::*;
use crate::tracing::info;

#[cfg(feature = "macros")]
pub use entrypoint_macros::entrypoint;

pub trait Entrypoint: Parser + EnvironmentVariableConfig + LoggingConfig {
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
        };
        info!("setup/config complete; executing entrypoint");
        function(entrypoint)
    }
}
impl<P: Parser> Entrypoint for P {}

pub trait LoggingConfig: Parser {
    fn configure_logging(self) -> Result<Self> {
        let format = tracing_subscriber::fmt::format();

        // #FIXME use try_init() instead?
        tracing_subscriber::fmt().event_format(format).init();

        Ok(self)
    }
}
impl<P: Parser> LoggingConfig for P {}

pub trait EnvironmentVariableConfig: Parser {
    /// user should/could override this
    /// order matters
    fn env_files(&self) -> Option<Vec<std::path::PathBuf>> {
        info!("env_files() default impl returns None");
        None
    }

    // #FIXME fn dump_env_vars(self) -> Self {}

    /// order matters - env, .env, passed paths
    /// don't override this
    fn process_env_files(self) -> Result<Self> {
        // do twice in case `env_files()` is dependant on supplied `.env`
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
impl<P: Parser> EnvironmentVariableConfig for P {}

#![forbid(unsafe_code)]
#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![warn(clippy::all, clippy::cargo, clippy::nursery, clippy::pedantic)]
#![warn(clippy::unwrap_used)]

////////////////////////////////////////////////////////////////////////////////
pub extern crate anyhow;
pub extern crate clap;
pub extern crate tracing;
pub extern crate tracing_subscriber;

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
    pub use crate::tracing::{debug, error, info, trace, warn, Level};

    pub use crate::tracing_subscriber;
    pub use crate::tracing_subscriber::fmt::SubscriberBuilder;

    pub use crate::DotEnvParser;
    pub use crate::Entrypoint;
    pub use crate::Logger;

    #[cfg(feature = "macros")]
    pub use crate::macros::*;
}

pub use crate::prelude::*;

////////////////////////////////////////////////////////////////////////////////
pub trait Entrypoint: Parser + DotEnvParser + Logger {
    fn additional_configuration(self) -> Result<Self> {
        Ok(self)
    }

    fn entrypoint<F, T>(self, function: F) -> Result<T>
    where
        F: FnOnce(Self) -> Result<T>,
    {
        let entrypoint = {
            {
                // use temp/local/default log subscriber until global is set by log_init()
                let _log = tracing::subscriber::set_default(self.log_subscriber().finish());

                self.process_dotenv_files()?.log_init()?
            }
            .additional_configuration()?
            .dump_env_vars()
        };

        info!("setup/config complete; executing entrypoint");
        function(entrypoint)
    }
}
impl<P: Parser + DotEnvParser + Logger> Entrypoint for P {}

////////////////////////////////////////////////////////////////////////////////
pub trait Logger: Parser {
    fn log_level(&self) -> Level {
        <Level as std::str::FromStr>::from_str("info")
            .expect("tracing::Level::from_str() invalid input")
    }

    fn log_subscriber(&self) -> SubscriberBuilder {
        let format = tracing_subscriber::fmt::format();

        tracing_subscriber::fmt()
            .event_format(format)
            .with_max_level(self.log_level())
    }

    fn log_init(self) -> Result<Self> {
        self.log_subscriber().init();

        Ok(self)
    }
}

////////////////////////////////////////////////////////////////////////////////
pub trait DotEnvParser: Parser {
    /// user should/could override this
    /// order matters
    fn dotenv_files(&self) -> Option<Vec<std::path::PathBuf>> {
        info!("dotenv_files() default impl returns None");
        None
    }

    /// #FIXME - doc
    /// warning: debug log_level can leak secrets
    fn dump_env_vars(self) -> Self {
        for (key, value) in std::env::vars() {
            debug!("{key}: {value}");
        }

        self
    }

    /// order matters - env, .env, passed paths
    /// don't override this
    fn process_dotenv_files(self) -> Result<Self> {
        // do twice in case `dotenv_files()` is dependant on `.env` supplied variable
        for _ in 0..=1 {
            let processed_found_dotenv = dotenvy::dotenv().map_or(Err(()), |file| {
                info!("dotenv::from_filename({})", file.display());
                Ok(())
            });

            // #FIXME - use map_or() here too?
            let processed_supplied_dotenv = if let Some(files) = self.dotenv_files() {
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

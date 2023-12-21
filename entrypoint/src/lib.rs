//! entrypoint: an app wrapper to eliminate startup boilerplate

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
    pub use crate::entrypoint_macros::DotEnvDefault;
    pub use crate::entrypoint_macros::LoggerDefault;
}

////////////////////////////////////////////////////////////////////////////////
pub mod prelude {
    pub use crate::anyhow;
    pub use crate::anyhow::Context;

    pub use crate::clap;
    pub use crate::clap::Parser;

    pub use crate::tracing;
    pub use crate::tracing::{debug, error, info, trace, warn};
    pub use crate::tracing_subscriber;

    pub use crate::DotEnvParser;
    pub use crate::Entrypoint;
    pub use crate::Logger;

    #[cfg(feature = "macros")]
    pub use crate::macros::*;
}

pub use crate::prelude::*;

////////////////////////////////////////////////////////////////////////////////
pub trait Entrypoint: clap::Parser + DotEnvParser + Logger {
    fn entrypoint<F, T>(self, function: F) -> anyhow::Result<T>
    where
        F: FnOnce(Self) -> anyhow::Result<T>,
    {
        let entrypoint = {
            {
                // use temp/local/default log subscriber until global is set by log_init()
                let _log = tracing::subscriber::set_default(self.log_subscriber().finish());

                self.process_dotenv_files()?;

                Self::parse() // parse again, dotenv might have defined some of the arg(env) fields
                    .process_dotenv_files()? // dotenv, again... same reason as above
                    .log_init()?
            }
        };
        info!("setup/config complete; executing entrypoint function");

        function(entrypoint)
    }
}
impl<P: clap::Parser + DotEnvParser + Logger> Entrypoint for P {}

////////////////////////////////////////////////////////////////////////////////
pub trait Logger: clap::Parser {
    fn log_level(&self) -> tracing::Level {
        tracing_subscriber::fmt::Subscriber::DEFAULT_MAX_LEVEL
            .into_level()
            .expect("invalid DEFAULT_MAX_LEVEL")
    }

    fn log_subscriber(&self) -> tracing_subscriber::fmt::SubscriberBuilder {
        let format = tracing_subscriber::fmt::format();

        tracing_subscriber::fmt()
            .event_format(format)
            .with_max_level(self.log_level())
    }

    fn log_init(self) -> anyhow::Result<Self> {
        if self.log_subscriber().try_init().is_err() {
            warn!("tracing_subscriber::try_init() failed");
        }

        info!(
            "init log level: {}",
            tracing_subscriber::filter::LevelFilter::current()
                .into_level()
                .expect("invalid LevelFilter::current()")
        );

        Ok(self)
    }
}

////////////////////////////////////////////////////////////////////////////////
pub trait DotEnvParser: clap::Parser {
    /// user should/could override this
    /// order matters
    fn additional_dotenv_files(&self) -> Option<Vec<std::path::PathBuf>> {
        None
    }

    fn dotenv_can_override(&self) -> bool {
        false
    }

    /// order matters - env, .env, passed paths
    /// don't override this
    fn process_dotenv_files(self) -> anyhow::Result<Self> {
        // do twice in case `additional_dotenv_files()` is dependant on `.env`
        for _ in 0..=1 {
            let processed_found_dotenv = {
                if self.dotenv_can_override() {
                    dotenvy::dotenv_override().map_or(Err(()), |file| {
                        info!("dotenv::from_filename_override({})", file.display());
                        Ok(())
                    })
                } else {
                    dotenvy::dotenv().map_or(Err(()), |file| {
                        info!("dotenv::from_filename({})", file.display());
                        Ok(())
                    })
                }
            };

            let processed_supplied_dotenv = self.additional_dotenv_files().map_or(Err(()), |files| {
                for file in files {
                    if self.dotenv_can_override() {
                        info!("dotenv::from_filename_override({})", file.display());
                        dotenvy::from_filename_override(file).or(Err(()))?;
                    } else {
                        info!("dotenv::from_filename({})", file.display());
                        dotenvy::from_filename(file).or(Err(()))?;
                    }
                }
                Ok(())
            });

            if processed_found_dotenv.is_err() && processed_supplied_dotenv.is_err() {
                info!("no dotenv file(s) found/processed");
                break;
            }
        }

        Ok(self)
    }
}

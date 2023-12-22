//! an (opinionated) app wrapper to eliminate main function boilerplate
//!
//! Eliminate boilerplate by smartly integrating:
//! * [`anyhow`](https://crates.io/crates/anyhow): for easy error handling
//! * [`clap`](https://crates.io/crates/clap): for easy CLI parsing
//! * [`dotenv`](https://crates.io/crates/dotenv): for easy environment variable management
//! * [`tracing`](https://crates.io/crates/tracing): for easy logging
//!
//! In lieu of `main()`, an [`entrypoint`] function is defined.
//!
//! Perfectly reasonable setup/config is done automagically.
//! More explicitly, the [`entrypoint`] function can be written as if:
//! * [`anyhow::Error`] is ready to propogate
//! * CLI have been parsed
//! * `.dotenv` files have already been processed and populated into the environment
//! * logging is ready to use
//!
//! Customization can be acheived by overriding various [trait](#traits) default implementations.
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
//!     // tracing & parsed clap struct are ready-to-use
//!     info!("entrypoint input args: {:#?}", args);
//!
//!     // env vars already have values from dotenv file(s)
//!     for (key, value) in std::env::vars() {
//!         println!("{key}: {value}");
//!     }
//!
//!     // easy error propagation
//!     Ok(())
//! }
//! ```
//!
//! # Feature Flags
//! Name     | Description                     | Default?
//! ---------|---------------------------------|---------
//! `macros` | Enables optional utility macros | Yes
//!
#![forbid(unsafe_code)]
#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![warn(clippy::all, clippy::cargo, clippy::nursery, clippy::pedantic)]
#![warn(clippy::unwrap_used)]

pub extern crate anyhow;
pub extern crate clap;
pub extern crate tracing;
pub extern crate tracing_subscriber;

#[cfg(feature = "macros")]
pub extern crate entrypoint_macros;

/// re-export [`entrypoint_macros`](https://crates.io/crates/entrypoint_macros)
#[cfg(feature = "macros")]
pub mod macros {
    pub use crate::entrypoint_macros::entrypoint;
    pub use crate::entrypoint_macros::DotEnvDefault;
    pub use crate::entrypoint_macros::LoggerDefault;
}

/// essential [traits](#traits) and re-exports
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

    /// control whether successive dotenv files can override already defined environment variables
    ///
    /// Default behavior is to not override.
    /// This preserves the stock/default [`dotenvy`] behavior.
    /// Override this default implementation as needed.
    ///
    /// # Examples
    /// ```
    /// # #[derive(clap::Parser)]
    /// # struct Args {}
    /// impl entrypoint::DotEnvParser for Args {
    ///     fn dotenv_can_override(&self) -> bool { true }
    /// }
    /// ```
    fn dotenv_can_override(&self) -> bool {
        false
    }

    /// process dotenv files and populate environment variables
    ///
    /// **You probably don't want to override this default implementation.**
    ///
    /// # Order Matters!
    /// Environment variables are processed/set in this order:
    /// 1. already defined in environment
    /// 2. `.env` file, if present
    /// 3. [`additional_dotenv_files`] supplied file(s) (sequentially as supplied)
    ///
    /// Keep in mind:
    /// * Depending on [`dotenv_can_override`], environment variables values may be the first *or* last processed/set.
    /// * [`additional_dotenv_files`] should be supplied in the order to be processed
    ///
    /// # Errors
    /// * processing an [`additional_dotenv_files`] supplied file fails
    fn process_dotenv_files(self) -> anyhow::Result<Self> {
        if self.dotenv_can_override() {
            dotenvy::dotenv_override()
                .map(|file| info!("dotenv::from_filename_override({})", file.display()))
        } else {
            dotenvy::dotenv().map(|file| info!("dotenv::from_filename({})", file.display()))
        }
        .map_err(|_| warn!("no .env file found"))
        .unwrap_or(()); // suppress, no .env is a valid use case

        self.additional_dotenv_files().map_or(Ok(()), |files| {
            // try all, so any/all failures will be in the log
            files.into_iter().fold(Ok(()), |accum, file| {
                let process = |res: Result<std::path::PathBuf, dotenvy::Error>, msg| {
                    res.map(|_| info!(msg)).map_err(|e| {
                        error!(msg);
                        e
                    })
                };

                if self.dotenv_can_override() {
                    process(
                        dotenvy::from_filename_override(file.clone()),
                        format!("dotenv::from_filename_override({})", file.display()),
                    )
                } else {
                    process(
                        dotenvy::from_filename(file.clone()),
                        format!("dotenv::from_filename({})", file.display()),
                    )
                }
                .and(accum)
            })
        })?; // bail if any of the additional_dotenv_files failed

        Ok(self)
    }
}

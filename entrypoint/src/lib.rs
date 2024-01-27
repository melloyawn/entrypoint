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
//! Customization can be acheived by overriding various [trait](crate#traits) default implementations.
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
//! fn entrypoint(args: Args) -> anyhow::Result<()> {
//!     // tracing & parsed clap struct are ready-to-use
//!     info!("entrypoint input args: {:#?}", args);
//!
//!     // env vars already have values from dotenv file(s)
//!     for (key, value) in std::env::vars() {
//!         println!("{key}: {value}");
//!     }
//!
//!     // easy error propagation w/ anyhow
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
    pub use crate::tracing::{debug, error, info, trace, warn, Subscriber};

    pub use crate::tracing_subscriber;
    pub use crate::tracing_subscriber::filter::LevelFilter;
    pub use crate::tracing_subscriber::fmt::{
        format::{Compact, Format, Full, Json, Pretty},
        FormatEvent, FormatFields, Layer, MakeWriter,
    };
    pub use crate::tracing_subscriber::prelude::*;
    pub use crate::tracing_subscriber::registry::LookupSpan;
    pub use crate::tracing_subscriber::reload;

    pub use crate::DotEnvParser;
    pub use crate::Entrypoint;
    pub use crate::Logger;

    #[cfg(feature = "macros")]
    pub use crate::macros::*;
}

pub use crate::prelude::*;

/// wrap a function with `main()` setup/initialization boilerplate
///
/// Refer to required [trait](crate#traits) bounds for more information and customization options.
///
/// # Examples
/// ```
/// # use entrypoint::prelude::*;
/// # #[derive(clap::Parser, DotEnvDefault, LoggerDefault)]
/// struct Args { }
///
/// // this function replaces `main()`
/// // this is the verbose/explicit way to define an entrypoint
/// // ...use the #[entrypoint::entrypoint] attribute macro instead
/// fn entrypoint(args: Args) -> anyhow::Result<()> {
///     Ok(())
/// }
///
/// // execute entrypoint from main
/// fn main() -> anyhow::Result<()> {
///     <Args as clap::Parser>::parse().entrypoint(entrypoint)
/// }
/// ```
pub trait Entrypoint: clap::Parser + DotEnvParser + Logger {
    /// run setup/configuration/initialization and execute supplied function
    ///
    /// **Don't override this default implementation.**
    ///
    /// # Errors
    /// * failure processing [`dotenv`](DotEnvParser) file(s)
    /// * failure configuring [logging](Logger)
    fn entrypoint<F, T>(self, function: F) -> anyhow::Result<T>
    where
        F: FnOnce(Self) -> anyhow::Result<T>,
    {
        let entrypoint = {
            // use temp/local/default log subscriber until global is set by initialize()
            let _log = tracing::subscriber::set_default(tracing_subscriber::fmt().finish());

            self.process_dotenv_files()?;

            Self::parse() // parse again, dotenv might have defined some of the arg(env) fields
                .process_dotenv_files()? // dotenv, again... same reason as above
                .initialize()?
        };
        info!("setup/config complete; executing entrypoint function");

        function(entrypoint)
    }
}
impl<P: clap::Parser + DotEnvParser + Logger> Entrypoint for P {}

/// automatic [`tracing`] & [`tracing_subscriber`] config/setup
///
/// Default implementations are what you'd expect.
/// If you don't need customization(s), use this [derive macro](entrypoint_macros::LoggerDefault).
///
/// # Examples
/// ```
/// # use entrypoint::prelude::*;
/// # #[derive(clap::Parser, DotEnvDefault)]
/// struct Args { }
///
/// impl entrypoint::Logger for Args { }
///
/// #[entrypoint::entrypoint]
/// fn entrypoint(args: Args) -> anyhow::Result<()> {
///     // logs are ready to use
///     info!("hello!");
/// #   Ok(())
/// }
/// ```
pub trait Logger: clap::Parser {
    /// define default [`tracing_subscriber`] [`LevelFilter`]
    ///
    /// Defaults to [`DEFAULT_MAX_LEVEL`](tracing_subscriber::fmt::Subscriber::DEFAULT_MAX_LEVEL).
    ///
    /// # Examples
    /// ```
    /// # use entrypoint::prelude::*;
    /// # #[derive(clap::Parser)]
    /// struct Args {
    ///     /// allow user to pass in debug level
    ///     #[arg(long)]
    ///     log_level: LevelFilter,
    /// }
    ///
    /// impl entrypoint::Logger for Args {
    ///     fn log_level(&self) -> LevelFilter {
    ///         self.log_level.clone()
    ///     }
    /// }
    /// ```
    fn log_level(&self) -> LevelFilter {
        tracing_subscriber::fmt::Subscriber::DEFAULT_MAX_LEVEL
    }

    /// define default [`tracing_subscriber`] [`Format`]
    ///
    /// Defaults to [`Format::default`].
    ///
    /// # Examples
    /// ```
    /// # use entrypoint::prelude::*;
    /// # #[derive(clap::Parser)]
    /// # struct Args { }
    /// impl entrypoint::Logger for Args {
    ///     fn log_format<S,N>(&self) -> impl FormatEvent<S,N> + Send + Sync + 'static
    ///     where
    ///         S: Subscriber + for<'a> LookupSpan<'a>,
    ///         N: for<'writer> FormatFields<'writer> + 'static,
    ///     {
    ///         Format::default().pretty()
    ///     }
    /// }
    /// ```
    fn log_format<S, N>(&self) -> impl FormatEvent<S, N> + Send + Sync + 'static
    where
        S: tracing::Subscriber + for<'a> LookupSpan<'a>,
        N: for<'writer> FormatFields<'writer> + 'static,
    {
        Format::default()
    }

    /// define default [`tracing_subscriber`] [`MakeWriter`]
    ///
    /// Defaults to [`std::io::stdout`].
    ///
    /// # Examples
    /// ```
    /// # use entrypoint::prelude::*;
    /// # #[derive(clap::Parser)]
    /// # struct Args { }
    /// impl entrypoint::Logger for Args {
    ///     fn log_writer(&self) -> impl for<'writer> MakeWriter<'writer> + Send + Sync + 'static {
    ///         std::io::stderr
    ///     }
    /// }
    /// ```
    fn log_writer(&self) -> impl for<'writer> MakeWriter<'writer> + Send + Sync + 'static {
        std::io::stdout
    }

    /// define [`tracing_subscriber`] [`Layer(s)`](Layer) to register
    ///
    /// **You probably don't want to override this default implementation.**
    /// Instead, override one of these other trait methods:
    /// * [Logger::log_level]
    /// * [Logger::log_format]
    /// * [Logger::log_writer]
    ///
    ///
    ///
    fn log_layers<S>(
        &self,
    ) -> Option<Vec<Box<dyn tracing_subscriber::Layer<S> + Send + Sync + 'static>>>
    where
        S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    {
        let (layer, reload) = reload::Layer::new(
            tracing_subscriber::fmt::Layer::default()
                .event_format(self.log_format())
                .with_writer(self.log_writer())
                .with_filter(self.log_level()),
        );

        let _ = reload.modify(|layer| *layer.filter_mut() = self.log_level());
        let _ = reload.modify(|layer| *layer.inner_mut().writer_mut() = self.log_writer());

        Some(vec![layer.boxed()])
    }

    /// setup and install the global tracing subscriber
    ///
    /// **You probably don't want to override this default implementation.**
    /// Instead, override one of these other trait methods:
    /// * [Logger::log_level]
    /// * [Logger::log_format]
    /// * [Logger::log_writer]
    /// * [Logger::log_layers]
    ///
    /// # Errors
    /// * [`tracing::subscriber::set_global_default`] was unsuccessful, likely because a global subscriber was already installed
    fn initialize(self) -> anyhow::Result<Self> {
        let subscriber = tracing_subscriber::Registry::default().with(self.log_layers());

        if tracing::subscriber::set_global_default(subscriber).is_err() {
            anyhow::bail!("tracing::subscriber::set_global_default failed");
        }

        info!(
            "log level: {}",
            LevelFilter::current()
                .into_level()
                .expect("invalid LevelFilter::current()")
        );

        Ok(self)
    }
}

/// automatic [`dotenv`](dotenvy) processing
///
/// Default implementations are what you'd expect.
/// If you don't need customization(s), use this [derive macro](entrypoint_macros::DotEnvDefault).
///
/// # Order Matters!
/// Environment variables are processed/set in this order:
/// 1. variables already defined in environment
/// 2. `.env` file, if present
/// 3. [`additional_dotenv_files`] supplied file(s) (sequentially, as supplied)
///
/// Keep in mind:
/// * Depending on [`dotenv_can_override`], environment variable values may be the first *or* last processed/set.
/// * [`additional_dotenv_files`] should be supplied in the order to be processed
///
/// # Examples
/// ```
/// # use entrypoint::prelude::*;
/// # #[derive(clap::Parser, LoggerDefault)]
/// struct Args { }
///
/// impl entrypoint::DotEnvParser for Args { }
///
/// #[entrypoint::entrypoint]
/// fn entrypoint(args: Args) -> anyhow::Result<()> {
///     // .env variables should now be in the environment
///     for (key, value) in std::env::vars() {
///         println!("{key}: {value}");
///     }
/// #   Ok(())
/// }
/// ```
///
/// [`additional_dotenv_files`]: DotEnvParser#method.additional_dotenv_files
/// [`dotenv_can_override`]: DotEnvParser#method.dotenv_can_override
pub trait DotEnvParser: clap::Parser {
    /// additional dotenv files to process
    ///
    /// Default behavior is to only use `.env` (i.e. no additional files).
    /// This preserves the stock/default [`dotenvy`] behavior.
    ///
    /// **[Order Matters!](DotEnvParser#order-matters)**
    ///
    /// # Examples
    /// ```
    /// # #[derive(clap::Parser)]
    /// struct Args {
    ///     /// allow user to pass in additional env files
    ///     #[arg(long)]
    ///     user_dotenv: Option<std::path::PathBuf>,
    /// }
    ///
    /// impl entrypoint::DotEnvParser for Args {
    ///     fn additional_dotenv_files(&self) -> Option<Vec<std::path::PathBuf>> {
    ///         self.user_dotenv.clone().map(|p| vec![p])
    ///     }
    /// }
    /// ```
    fn additional_dotenv_files(&self) -> Option<Vec<std::path::PathBuf>> {
        None
    }

    /// whether successive dotenv files can override already defined environment variables
    ///
    /// Default behavior is to not override.
    /// This preserves the stock/default [`dotenvy`] behavior.
    ///
    /// **[Order Matters!](DotEnvParser#order-matters)**
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

    /// process dotenv files and populate variables into the environment
    ///
    /// **You probably don't want to override this default implementation.**
    ///
    /// **[Order Matters!](DotEnvParser#order-matters)**
    ///
    /// # Errors
    /// * failure processing an [`additional_dotenv_files`] supplied file
    ///
    /// [`additional_dotenv_files`]: DotEnvParser#method.additional_dotenv_files
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

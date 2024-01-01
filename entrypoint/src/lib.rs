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
    pub use crate::tracing::{debug, error, info, trace, warn};

    pub use crate::tracing_subscriber;
    pub use crate::tracing_subscriber::filter::LevelFilter;
    pub use crate::tracing_subscriber::fmt::{format::Format, FormatEvent, Layer};
    pub use crate::tracing_subscriber::prelude::*;
    pub use crate::tracing_subscriber::registry::LookupSpan;
    pub use crate::tracing_subscriber::util::SubscriberInitExt;
    pub use crate::tracing_subscriber::Registry;

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
            {
                // use temp/local/default log subscriber until global is set by log_init()
                let _log = tracing::subscriber::set_default(tracing_subscriber::fmt().finish());

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
/// // since we're using the default impls
/// //   #[derive(LoggerDefault)] on Args would have been more suitable
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
    ///     fn log_format(&self) -> Format {
    ///         // non-default format
    ///         Format::default().pretty()
    ///     }
    /// }
    /// ```
    fn log_format(&self) -> Format {
        todo!("#FIXME this isn't being used anywhere"); //#FIXME
        Format::default()
    }

    /// define [`tracing_subscriber`] [`Layer(s)`](Layer) to register
    ///
    /// Defaults to [`Layer::default`] with:
    /// * [filtering] defined by [Logger::log_level] value.
    /// * [formatting] defined by [Logger::log_format] value.
    ///
    /// override this default implementation if:
    /// * mulitple layers are required
    /// * reload references are required #FIXME(explain/example; link reload)
    ///
    // #FIXME    /// # Examples
    // #FIXME    /// ```
    // #FIXME    /// # #[derive(clap::Parser)]
    // #FIXME    /// # struct Args { }
    // #FIXME    /// impl entrypoint::Logger for Args {
    // #FIXME    ///     fn log_subscriber_builder(&self) -> SubscriberBuilder {
    // #FIXME    ///         // use a non-default format
    // #FIXME    ///         tracing_subscriber::fmt().pretty().with_thread_names(true)
    // #FIXME    ///     }
    // #FIXME    /// }
    // #FIXME    /// ```
    /// [filtering]: tracing_subscriber::Layer::with_filter
    /// [formatting]: tracing_subscriber::Layer::event_format
    fn log_layers<S>(
        &self,
    ) -> Option<Vec<Box<dyn tracing_subscriber::Layer<S> + Send + Sync + 'static>>>
    where
        S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    {
        let format = Format::default(); //#FIXME
        let layer = tracing_subscriber::fmt::layer()
            .event_format(format)
            .with_filter(self.log_level())
            .boxed();

        Some(vec![layer])
    }

    /// define the [`tracing_subscriber`] [`Registry`]
    ///
    /// Defaults to [`Registry::default`].
    ///
    /// override this default implementation if:
    /// * reload references are required #FIXME(explain/example; link reload)
    ///
    // #FIXME /// # Examples
    // #FIXME /// ```
    // #FIXME /// ```
    fn log_registry(&self) -> Registry {
        Registry::default()
    }

    /// init and install the global tracing subscriber
    ///
    /// **You probably don't want to override this default implementation.**
    /// Instead, override [Logger::log_level], [Logger::log_format], [Logger::log_layers], or [Logger::log_registry].
    ///
    /// # Errors
    /// * [`tracing_subscriber::util::SubscriberInitExt::try_init`] was unsuccessful, likely because a global subscriber was already installed
    fn log_init(self) -> anyhow::Result<Self> {
        let subscriber = self.log_registry().with(self.log_layers());

        if subscriber.try_init().is_err() {
            anyhow::bail!("tracing_subscriber::util::SubscriberInitExt::try_init() failure");
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
/// // since we're using the default impls
/// //   #[derive(DotEnvDefault)] on Args would have been more suitable
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

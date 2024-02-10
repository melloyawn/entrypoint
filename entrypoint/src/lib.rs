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
//! More explicitly, the [`entrypoint`](Entrypoint::entrypoint) function can be written as if:
//! * [`anyhow::Error`] is ready to propogate
//! * CLI have been parsed
//! * `.dotenv` files have already been processed and populated into the environment
//! * logging is ready to use
//!
//! Customization can be achieved by overriding various [trait](crate#traits) default implementations
//! (or preferably/more-typically by using the provided [attribute macros](macros)).
//!
//! # Examples
//! ```
//! use entrypoint::prelude::*;
//!
//! #[derive(clap::Parser, DotEnvDefault, LoggerDefault, Debug)]
//! #[log_format(pretty)]
//! #[log_level(entrypoint::LevelFilter::DEBUG)]
//! #[log_writer(std::io::stdout)]
//! struct Args {}
//!
//! // this function replaces `main`
//! #[entrypoint::entrypoint]
//! fn main(args: Args) -> anyhow::Result<()> {
//!     // tracing & parsed clap struct are ready-to-use
//!     debug!("entrypoint input args: {:#?}", args);
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
//! Name       | Description                     | Default?
//! -----------|---------------------------------|---------
//! [`macros`] | Enables optional utility macros | Yes
//!

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
    pub use crate::tracing::{
        debug, enabled, error, event, info, instrument, trace, warn, Level, Subscriber,
    };
    pub use crate::tracing::{debug_span, error_span, info_span, span, trace_span, warn_span};

    pub use crate::tracing_subscriber;
    pub use crate::tracing_subscriber::filter::LevelFilter;
    pub use crate::tracing_subscriber::fmt::{
        format::{Compact, Format, Full, Json, Pretty},
        FormatEvent, FormatFields, Layer, MakeWriter,
    };
    pub use crate::tracing_subscriber::prelude::*;
    pub use crate::tracing_subscriber::registry::LookupSpan;
    pub use crate::tracing_subscriber::reload;
    pub use crate::tracing_subscriber::Registry;

    pub use crate::Entrypoint;
    pub use crate::{DotEnvParser, DotEnvParserConfig};
    pub use crate::{Logger, LoggerConfig};

    #[cfg(feature = "macros")]
    pub use crate::macros::*;
}

pub use crate::prelude::*;

/// blanket implementation to wrap a function with "`main()`" setup/initialization boilerplate
///
/// Refer to required [trait](crate#traits) bounds for more information and customization options.
///
/// # Examples
/// **Don't copy this code example. Use the [`macros::entrypoint`] attribute macro instead.**
/// ```
/// # use entrypoint::prelude::*;
/// # #[derive(clap::Parser, DotEnvDefault, LoggerDefault)]
/// struct Args {}
///
/// // this function "replaces" `main()`
/// fn entrypoint(args: Args) -> anyhow::Result<()> {
///     Ok(())
/// }
///
/// // execute entrypoint from main
/// fn main() -> anyhow::Result<()> {
///     <Args as clap::Parser>::parse().entrypoint(entrypoint)
/// }
/// ```
pub trait Entrypoint: clap::Parser + DotEnvParserConfig + LoggerConfig {
    /// run setup/configuration/initialization and execute supplied function
    ///
    /// Customize if/as needed with the other entrypoint [traits](crate#traits).
    ///
    /// # Errors
    /// * failure processing [`dotenv`](DotEnvParserConfig) file(s)
    /// * failure configuring [logging](LoggerConfig)
    fn entrypoint<F, T>(self, function: F) -> anyhow::Result<T>
    where
        F: FnOnce(Self) -> anyhow::Result<T>,
    {
        let entrypoint = {
            // use temp/local/default log subscriber until global is set by log_init()
            let _log = tracing::subscriber::set_default(tracing_subscriber::fmt().finish());

            self.process_dotenv_files()?;

            Self::parse() // parse again, dotenv might have defined some of the arg(env) fields
                .process_dotenv_files()? // dotenv, again... same reason as above
                .log_init(None)?
        };
        info!("setup/config complete; executing entrypoint function");

        function(entrypoint)
    }
}
impl<T: clap::Parser + DotEnvParserConfig + LoggerConfig> Entrypoint for T {}

/// automatic [`tracing`] & [`tracing_subscriber`] configuration
///
/// Available configuration for the [`Logger`] trait.
///
/// Default implementations are what you'd expect.
/// Use this [derive macro](macros::LoggerDefault) for typical use cases.
///
/// # Examples
/// ```
/// # use entrypoint::prelude::*;
/// # #[derive(clap::Parser, DotEnvDefault)]
/// #[derive(LoggerDefault)]
/// #[log_format(full)]
/// #[log_level(entrypoint::LevelFilter::DEBUG)]
/// #[log_writer(std::io::stdout)]
/// struct Args {}
///
/// #[entrypoint::entrypoint]
/// fn main(args: Args) -> anyhow::Result<()> {
///     // logs are ready to use
///     info!("hello!");
/// #   Ok(())
/// }
/// ```
/// For advanced customization requirements, refer to [`LoggerConfig::bypass_log_init`].
pub trait LoggerConfig: clap::Parser {
    /// hook to disable/enable automatic initialization
    ///
    /// This disrupts automatic initialization so that completely custom [`Layer`]s can be provided to [`Logger::log_init`].
    /// This is intended only for advanced use cases, such as:
    /// 1. multiple [`Layer`]s are required
    /// 2. a [reload handle](tracing_subscriber::reload::Handle) needs to be kept accessible
    ///
    /// Default behvaior ([`false`]) is to call [`Logger::log_init`] on startup and
    /// register the default layer provided by [`LoggerConfig::default_log_layer`].
    ///
    /// Overriding this to [`true`] will **not** automatically call [`Logger::log_init`] on startup.
    /// All other defaults provided by the [`LoggerConfig`] trait methods are ignored.
    /// The application is then **required** to directly call [`Logger::log_init`] with explicitly provided layer(s).
    ///
    /// # Examples
    /// ```
    /// # use entrypoint::prelude::*;
    /// # #[derive(clap::Parser, DotEnvDefault)]
    /// struct Args {}
    ///
    /// impl entrypoint::LoggerConfig for Args {
    ///     fn bypass_log_init(&self) -> bool { true }
    /// }
    ///
    /// #[entrypoint::entrypoint]
    /// fn main(args: Args) -> anyhow::Result<()> {
    ///     // logging hasn't been configured yet
    ///     assert!(!enabled!(entrypoint::Level::ERROR));
    ///
    ///     // must manually config/init logging
    ///     let (layer, reload_handle) = reload::Layer::new(
    ///         tracing_subscriber::fmt::Layer::default()
    ///             .event_format(args.default_log_format())
    ///             .with_writer(args.default_log_writer())
    ///             .with_filter(args.default_log_level()),
    ///     );
    ///     let args = args.log_init(Some(vec![layer.boxed()]))?;
    ///
    ///     // OK... now logging should work
    ///     assert!( enabled!(entrypoint::Level::ERROR));
    ///     assert!(!enabled!(entrypoint::Level::TRACE));
    ///
    ///     // we've maintained direct access to the layer and reload handle
    ///     let _ = reload_handle.modify(|layer| *layer.filter_mut() = entrypoint::LevelFilter::TRACE);
    ///     assert!( enabled!(entrypoint::Level::TRACE));
    /// #   Ok(())
    /// }
    /// ```
    fn bypass_log_init(&self) -> bool {
        false
    }

    /// define the default [`tracing_subscriber`] [`LevelFilter`]
    ///
    /// Defaults to [`DEFAULT_MAX_LEVEL`](tracing_subscriber::fmt::Subscriber::DEFAULT_MAX_LEVEL).
    ///
    /// This can be easily set with convenience [`macros`](macros::LoggerDefault#attributes).
    ///
    /// # Examples
    /// ```
    /// # use entrypoint::prelude::*;
    /// # #[derive(clap::Parser)]
    /// struct Args {
    ///     /// allow user to pass in debug level
    ///     #[arg(long)]
    ///     default_log_level: LevelFilter,
    /// }
    ///
    /// impl entrypoint::LoggerConfig for Args {
    ///     fn default_log_level(&self) -> LevelFilter {
    ///         self.default_log_level.clone()
    ///     }
    /// }
    /// ```
    fn default_log_level(&self) -> LevelFilter {
        tracing_subscriber::fmt::Subscriber::DEFAULT_MAX_LEVEL
    }

    /// define the default [`tracing_subscriber`] [`Format`]
    ///
    /// Defaults to [`Format::default`].
    ///
    /// This can be easily set with convenience [`macros`](macros::LoggerDefault#attributes).
    ///
    /// # Examples
    /// ```
    /// # use entrypoint::prelude::*;
    /// # #[derive(clap::Parser)]
    /// # struct Args {}
    /// impl entrypoint::LoggerConfig for Args {
    ///     fn default_log_format<S,N>(&self) -> impl FormatEvent<S,N> + Send + Sync + 'static
    ///     where
    ///         S: Subscriber + for<'a> LookupSpan<'a>,
    ///         N: for<'writer> FormatFields<'writer> + 'static,
    ///     {
    ///         Format::default().pretty()
    ///     }
    /// }
    /// ```
    fn default_log_format<S, N>(&self) -> impl FormatEvent<S, N> + Send + Sync + 'static
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
        N: for<'writer> FormatFields<'writer> + 'static,
    {
        Format::default()
    }

    /// define the default [`tracing_subscriber`] [`MakeWriter`]
    ///
    /// Defaults to [`std::io::stdout`].
    ///
    /// This can be easily set with convenience [`macros`](macros::LoggerDefault#attributes).
    ///
    /// # Examples
    /// ```
    /// # use entrypoint::prelude::*;
    /// # #[derive(clap::Parser)]
    /// # struct Args {}
    /// impl entrypoint::LoggerConfig for Args {
    ///     fn default_log_writer(&self) -> impl for<'writer> MakeWriter<'writer> + Send + Sync + 'static {
    ///         std::io::stderr
    ///     }
    /// }
    /// ```
    fn default_log_writer(&self) -> impl for<'writer> MakeWriter<'writer> + Send + Sync + 'static {
        std::io::stdout
    }

    /// define the default [`tracing_subscriber`] [`Layer`] to register
    ///
    /// This method uses the defaults defined by [`LoggerConfig`] methods and composes a default [`Layer`] to register.
    ///
    /// **You ***probably*** don't want to override this default implementation.**
    /// 1. For standard customization, override these other trait methods:
    ///    * [`LoggerConfig::default_log_level`]
    ///    * [`LoggerConfig::default_log_format`]
    ///    * [`LoggerConfig::default_log_writer`]
    /// 2. Minor/static customization(s) ***can*** be achieved by overriding this method...
    ///    though this might warrant moving to the 'advanced requirements' option below.
    /// 3. Otherwise, for advanced requirements, refer to [`LoggerConfig::bypass_log_init`].
    fn default_log_layer(
        &self,
    ) -> Box<dyn tracing_subscriber::Layer<Registry> + Send + Sync + 'static> {
        let (layer, _) = reload::Layer::new(
            tracing_subscriber::fmt::Layer::default()
                .event_format(self.default_log_format())
                .with_writer(self.default_log_writer())
                .with_filter(self.default_log_level()),
        );

        layer.boxed()
    }
}

/// blanket implementation for automatic [`tracing`] & [`tracing_subscriber`] initialization
///
/// Refer to [`LoggerConfig`] for configuration options.
pub trait Logger: LoggerConfig {
    /// register the supplied layers with the global tracing subscriber
    ///
    /// Default behvaior is to automatically (on startup) register the layer provided by [`LoggerConfig::default_log_layer`].
    ///
    /// This automatic setup/config can be disabled with [`LoggerConfig::bypass_log_init`].
    /// When bypassed, **[`Logger::log_init`] must be manually/directly called from the application.**
    /// This is an advanced use case. Refer to [`LoggerConfig::bypass_log_init`] for more details.
    ///
    /// # Errors
    /// * [`tracing::subscriber::set_global_default`] was unsuccessful, likely because a global subscriber was already installed
    fn log_init(
        self,
        layers: Option<Vec<Box<dyn tracing_subscriber::Layer<Registry> + Send + Sync + 'static>>>,
    ) -> anyhow::Result<Self> {
        let layers = match (self.bypass_log_init(), &layers) {
            (false, Some(_)) => {
                anyhow::bail!("bypass_log_init() is false, but layers were passed into log_init()");
            }
            (false, None) => Some(vec![self.default_log_layer()]),
            (true, _) => layers,
        };

        if layers.is_some()
            && tracing_subscriber::registry()
                .with(layers)
                .try_init()
                .is_err()
        {
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
impl<T: LoggerConfig> Logger for T {}

/// automatic [`dotenv`](dotenvy) processing configuration
///
/// Available configuration for the [`DotEnvParser`] trait.
///
/// Default implementations are what you'd expect.
/// Use this [derive macro](macros::DotEnvDefault) for typical use cases.
///
/// # Order Matters!
/// Environment variables are processed/set in this order:
/// 1. Preexisting variables already defined in environment.
/// 2. The `.env` file, if present.
/// 3. [`additional_dotenv_files`] supplied file(s) (sequentially, as supplied).
///
/// Keep in mind:
/// * Depending on [`dotenv_can_override`], environment variable values may be the first *or* last processed/set.
/// * [`additional_dotenv_files`] should be supplied in the order to be processed.
///
/// # Examples
/// ```
/// # use entrypoint::prelude::*;
/// # #[derive(clap::Parser, LoggerDefault)]
/// #[derive(DotEnvDefault)]
/// struct Args {}
///
/// #[entrypoint::entrypoint]
/// fn main(args: Args) -> anyhow::Result<()> {
///     // .env variables should now be in the environment
///     for (key, value) in std::env::vars() {
///         println!("{key}: {value}");
///     }
/// #   Ok(())
/// }
/// ```
/// [`additional_dotenv_files`]: DotEnvParserConfig#method.additional_dotenv_files
/// [`dotenv_can_override`]: DotEnvParserConfig#method.dotenv_can_override
pub trait DotEnvParserConfig: clap::Parser {
    /// additional dotenv files to process
    ///
    /// Default behavior is to only use `.env` (i.e. no additional files).
    /// This preserves the stock/default [`dotenvy`] behavior.
    ///
    /// **[Order Matters!](DotEnvParserConfig#order-matters)**
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
    /// impl entrypoint::DotEnvParserConfig for Args {
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
    /// **[Order Matters!](DotEnvParserConfig#order-matters)**
    ///
    /// # Examples
    /// ```
    /// # #[derive(clap::Parser)]
    /// # struct Args {}
    /// impl entrypoint::DotEnvParserConfig for Args {
    ///     fn dotenv_can_override(&self) -> bool { true }
    /// }
    /// ```
    fn dotenv_can_override(&self) -> bool {
        false
    }
}

/// blanket implementation for automatic [`dotenv`](dotenvy) processing
///
/// Refer to [`DotEnvParserConfig`] for configuration options.
pub trait DotEnvParser: DotEnvParserConfig {
    /// process dotenv files and populate variables into the environment
    ///
    /// This will run automatically at startup.
    ///
    /// **[Order Matters!](DotEnvParserConfig#order-matters)**
    ///
    /// # Errors
    /// * failure processing an [`DotEnvParserConfig::additional_dotenv_files`] supplied file
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
            #[allow(clippy::manual_try_fold)]
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
impl<T: DotEnvParserConfig> DotEnvParser for T {}

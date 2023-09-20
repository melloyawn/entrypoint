#![forbid(unsafe_code)]

pub use anyhow;
pub use clap;
pub use tracing;

use anyhow::Result;
use clap::Parser;
use tracing::info;

pub trait Entrypoint: Parser + EnvironmentVariableConfig + LoggingConfig {
    fn additional_configuration(self) -> Result<Self> {
        Ok(self)
    }

    fn entrypoint<F>(self, function: F) -> Result<()>
    where
        F: FnOnce(Self) -> Result<()>,
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
            let processed_found_dotenv = if let Ok(file) = dotenvy::dotenv() {
                info!("dotenv::from_filename({})", file.display());
                Ok(())
            } else {
                Err(())
            };

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

//! override default trait impls w/ CLI args

use entrypoint::prelude::*;

/// input args are minimal... use dotenv files to define stuff
#[derive(entrypoint::clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// additional dotenv files to process; order matters!
    #[arg(short, long, num_args = 1..)]
    pub dotenv_files: Option<Vec<std::path::PathBuf>>,

    /// allow successive dotenv files to override previous ones
    #[arg(short, long, env, default_value_t = false)]
    pub allow_dotenv_overrides: bool,
}

impl DotEnvParser for Args {
    /// use value passed in via input [`Args`]
    fn additional_dotenv_files(&self) -> Option<Vec<std::path::PathBuf>> {
        self.dotenv_files.clone()
    }

    /// use value passed in via input [`Args`]
    fn dotenv_can_override(&self) -> bool {
        self.allow_dotenv_overrides
    }
}

impl Logger for Args {
    /// use value of env::var(LOG_LEVEL) (probably set via dotenv)
    /// default to "info" if undefined
    fn log_level(&self) -> entrypoint::tracing_subscriber::filter::LevelFilter {
        <entrypoint::tracing::Level as std::str::FromStr>::from_str(
            std::env::var("LOG_LEVEL")
                .unwrap_or(String::from("info"))
                .as_str(),
        )
        .expect("failed to parse Level")
        .into()
    }
}

#[entrypoint::entrypoint]
fn entrypoint(_args: Args) -> entrypoint::anyhow::Result<()> {
    info!("dumping env vars...");

    for (key, val) in std::env::vars() {
        info!("{key}: {val}");
    }

    trace!("this is a trace");
    debug!("this is a debug");
    info!("this is an info");
    warn!("this is a warn");
    error!("this is an error");

    Ok(())
}

//! a "real-world" example using axum

use axum::{response::Html, routing::get, Router};
use entrypoint::prelude::*;

/// input args; minimal... use dotenv files for other stuff
#[derive(entrypoint::clap::Parser, Debug)]
#[command(author, version)]
#[command(about = "run example axum server")]
#[command(
    after_help = "Note: running in a devel env probably requires: --allow-dotenv-overrides --dotenv-files .dev"
)]
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
    fn dotenv_files(&self) -> Option<Vec<std::path::PathBuf>> {
        self.dotenv_files.clone()
    }

    /// use value passed in via input [`Args`]
    fn dotenv_can_override(&self) -> bool {
        self.allow_dotenv_overrides
    }
}

impl Logger for Args {
    /// use value of env::var(LOG_LEVEL)
    /// defaults to "info"
    fn log_level(&self) -> entrypoint::tracing::Level {
        <entrypoint::tracing::Level as std::str::FromStr>::from_str(
            std::env::var("LOG_LEVEL")
                .unwrap_or(String::from("info"))
                .as_str(),
        )
        .expect("failed to parse Level")
    }
}

/// print to different log levels, return hello world
async fn log_test() -> Html<&'static str> {
    trace!("trace");
    debug!("debug");
    info!("info");
    warn!("warn");
    error!("error");
    Html("hello world")
}

/// server mainloop
#[tokio::main]
#[entrypoint::entrypoint]
async fn entrypoint(_args: Args) -> entrypoint::anyhow::Result<()> {
    let addr: std::net::SocketAddr = {
        format!(
            "{ip}:{port}",
            ip = std::env::var("IP").expect("env::var(IP)"),
            port = std::env::var("PORT").expect("env::var(PORT)"),
        )
        .parse()
        .expect("failed to parse SocketAddr")
    };

    let app = Router::new().route("/", get(log_test));

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("TcpListener::bind failed");
    info!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .expect("axum::serve failed");

    Ok(())
}

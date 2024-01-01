//! test w/ async/tokio

use entrypoint::prelude::*;

#[derive(entrypoint::clap::Parser, DotEnvDefault, LoggerDefault, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}

#[tokio::main]
#[entrypoint::entrypoint]
#[test]
async fn entrypoint(_args: Args) -> entrypoint::anyhow::Result<()> {
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    Ok(())
}

//! make sure async/tokio works

use entrypoint::prelude::*;

#[derive(entrypoint::clap::Parser, DotEnvDefault, LoggerDefault, Debug)]
#[log_writer(std::io::sink)]
#[command(author, version, about, long_about = None)]
struct Args {}

#[tokio::main]
#[entrypoint::entrypoint]
#[test]
async fn entrypoint(_args: Args) -> entrypoint::anyhow::Result<()> {
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    assert!(enabled!(entrypoint::Level::INFO));
    Ok(())
}

//! entrypoint w/ tokio usage example

use entrypoint::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    debug: bool,
}

#[entrypoint::entrypoint]
#[tokio::main]
async fn entrypoint(args: Args) -> entrypoint::Result<()> {
    entrypoint::tracing::info!("in tokio::main({:?})", args);
    Ok(())
}

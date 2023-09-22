//! a simple entrypoint usage example

// import entrypoint::prelude for the essentials
use entrypoint::prelude::*;

/// define a [`clap`] struct
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    debug: bool,
}

/// define an entrypoint function
fn run_app(args: Args) -> entrypoint::anyhow::Result<()> {
    entrypoint::tracing::info!("run_app({:?})", args);
    Ok(())
}

/// define the main function
fn main() -> entrypoint::anyhow::Result<()> {
    // call entrypoint from the [`clap`] struct
    Args::parse().entrypoint(run_app)
}

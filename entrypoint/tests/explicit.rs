//! entrypoint explicit usage example
//! i.e. not using the macros

use entrypoint::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    debug: bool,
}

// define an entrypoint function
fn entrypoint(args: Args) -> entrypoint::Result<()> {
    entrypoint::tracing::info!("in entrypoint({:?})", args);
    Ok(())
}

// define the main function
fn main() -> entrypoint::Result<()> {
    // call entrypoint from the [`clap`] struct
    Args::parse().entrypoint(entrypoint)
}

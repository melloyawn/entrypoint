//! example using default trait impls

use entrypoint::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    debug: bool,
}
impl entrypoint::DotEnvParser for Args {}
impl entrypoint::Logger for Args {}

// define an entrypoint function
fn entrypoint(args: Args) -> entrypoint::Result<()> {
    entrypoint::tracing::info!("in entrypoint({:?})", args);

    assert_eq!(args.log_level(), entrypoint::Level::INFO);

    Ok(())
}

// define the main function
#[test] // normally would not need
fn main() -> entrypoint::Result<()> {
    // call entrypoint from the [`clap`] struct
    Args::parse().entrypoint(entrypoint)
}

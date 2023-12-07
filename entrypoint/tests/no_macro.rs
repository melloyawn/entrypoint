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
    Ok(())
}

// define the main function
fn main() -> entrypoint::Result<()> {
    // call entrypoint from the [`clap`] struct
    Args::parse().entrypoint(entrypoint)
}

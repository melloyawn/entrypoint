//! example of typical (default/macro) usage (w/ tokio)

use entrypoint::prelude::*;

#[derive(Parser, DotEnvDefault, LoggerDefault, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    debug: bool,
}

#[entrypoint::entrypoint]
#[tokio::main]
#[test] // normally would not need
async fn entrypoint(args: Args) -> entrypoint::Result<()> {
    entrypoint::tracing::info!("in tokio::main({:?})", args);

    assert_eq!(args.log_level(), entrypoint::Level::INFO);

    Ok(())
}

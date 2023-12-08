//! example of typical (default/macro) usage

use entrypoint::prelude::*;

#[derive(Parser, DotEnvDefault, LoggerDefault, Debug)]
#[log_level(entrypoint::Level::DEBUG)] // set a non-default log level
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    debug: bool,
}

#[entrypoint::entrypoint]
#[test] // normally would not need
fn entrypoint(args: Args) -> entrypoint::Result<()> {
    entrypoint::tracing::info!("in entrypoint({:?})", args);

    // check log_level derive attribute
    assert_eq!(args.log_level(), entrypoint::Level::DEBUG);

    Ok(())
}

//! derive macros w/ defaults

use entrypoint::prelude::*;
mod common;

#[derive(entrypoint::clap::Parser, DotEnvDefault, LoggerDefault, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}

#[entrypoint::entrypoint]
#[test]
fn entrypoint(args: Args) -> entrypoint::anyhow::Result<()> {
    assert!(args.additional_dotenv_files().is_none());

    common::using_prod_env()?;

    common::verify_log_level(
        &args,
        &entrypoint::tracing_subscriber::filter::LevelFilter::INFO,
    )?; // default

    // #FIXME - how to check format?
    // #FIXME - how to check writer?

    Ok(())
}

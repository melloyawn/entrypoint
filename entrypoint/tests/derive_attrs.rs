//! derive macros + set log_level attribute

use entrypoint::prelude::*;
mod common;

#[derive(entrypoint::clap::Parser, DotEnvDefault, LoggerDefault, Debug)]
#[log_format(json)]
#[log_level(entrypoint::tracing_subscriber::filter::LevelFilter::DEBUG)]
#[log_writer(std::io::stderr)]
#[command(author, version, about, long_about = None)]
struct Args {}

#[entrypoint::entrypoint]
#[test]
fn entrypoint(args: Args) -> entrypoint::anyhow::Result<()> {
    assert!(args.additional_dotenv_files().is_none());

    common::using_prod_env()?;

    common::verify_log_level(
        &args,
        &entrypoint::tracing_subscriber::filter::LevelFilter::DEBUG,
    )?; // log_level attribute, not default

    // #FIXME - how to check format?
    // #FIXME - how to check writer?

    Ok(())
}

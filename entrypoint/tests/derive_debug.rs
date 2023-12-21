//! derive macros + set log_level attribute

use entrypoint::prelude::*;
mod common;

#[derive(entrypoint::clap::Parser, DotEnvDefault, LoggerDefault, Debug)]
#[log_level(entrypoint::tracing::Level::DEBUG)]
#[command(author, version, about, long_about = None)]
struct Args {}

#[entrypoint::entrypoint]
#[test]
fn entrypoint(args: Args) -> entrypoint::anyhow::Result<()> {
    assert!(args.additional_dotenv_files().is_none());

    common::using_prod_env()?;

    common::verify_log_level(&args, &entrypoint::tracing::Level::DEBUG)?; // log_level attribute, not default

    Ok(())
}

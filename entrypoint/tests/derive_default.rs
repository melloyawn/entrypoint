//! derive macros

use entrypoint::prelude::*;
mod common;

#[derive(entrypoint::clap::Parser, DotEnvDefault, LoggerDefault, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}

#[entrypoint::entrypoint]
#[test]
fn entrypoint(args: Args) -> entrypoint::anyhow::Result<()> {
    assert!(args.dotenv_files().is_none()); // no extra dotenv

    common::using_prod_env()?;

    common::verify_log_level(&args, &entrypoint::tracing::Level::INFO)?; // default

    Ok(())
}

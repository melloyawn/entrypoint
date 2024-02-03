//! verbose way, no macros
//! probably don't use this as any sort of example

use entrypoint::prelude::*;
mod common;

#[derive(entrypoint::clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}
impl entrypoint::DotEnvParser for Args {}
impl entrypoint::Logger for Args {}

/// entrypoint function
fn entrypoint(args: Args) -> entrypoint::anyhow::Result<()> {
    assert!(args.additional_dotenv_files().is_none());

    common::using_prod_env()?;

    common::verify_log_level(
        &args,
        &entrypoint::tracing_subscriber::filter::LevelFilter::INFO,
    )?; // default

    Ok(())
}

/// main function
#[test]
fn main() -> entrypoint::anyhow::Result<()> {
    <Args as entrypoint::clap::Parser>::parse().entrypoint(entrypoint)
}

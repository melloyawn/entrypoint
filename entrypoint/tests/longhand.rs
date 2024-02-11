//! verbose way, no macros... not a good usage example

use entrypoint::prelude::*;
mod common;

impl DotEnvParserConfig for common::Args {}

/// entrypoint function
fn entrypoint(args: common::Args) -> entrypoint::anyhow::Result<()> {
    assert!(args.additional_dotenv_files().is_none());

    common::using_prod_env()?;

    common::verify_log_level(
        &args,
        &entrypoint::tracing_subscriber::filter::LevelFilter::WARN,
    )?; // default

    Ok(())
}

/// main function
#[test]
fn main() -> entrypoint::anyhow::Result<()> {
    assert!(!enabled!(entrypoint::Level::ERROR));
    <common::Args as entrypoint::clap::Parser>::parse().entrypoint(entrypoint)
}

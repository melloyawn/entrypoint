//! use both .env and .dev; allow .dev to override

use entrypoint::prelude::*;
mod common;

impl DotEnvParserConfig for common::Args {
    fn additional_dotenv_files(&self) -> Option<Vec<std::path::PathBuf>> {
        Some(vec![std::path::PathBuf::from(".dev")])
    }

    fn dotenv_can_override(&self) -> bool {
        true
    }
}

#[entrypoint::entrypoint]
#[test]
fn entrypoint(args: common::Args) -> entrypoint::anyhow::Result<()> {
    common::using_both_yes_override()?;

    common::verify_log_level(
        &args,
        &entrypoint::tracing_subscriber::filter::LevelFilter::DEBUG,
    )?;

    Ok(())
}

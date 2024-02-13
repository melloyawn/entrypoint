//! derive macros + set log_level attribute

use entrypoint::prelude::*;
mod common;

#[derive(entrypoint::clap::Parser, DotEnvDefault, LoggerDefault, Debug)]
#[log_format(json)]
#[log_level(entrypoint::tracing_subscriber::filter::LevelFilter::DEBUG)]
#[log_writer(common::global_writer)]
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
    )?; // check log_level attribute, not default

    common::OUTPUT_BUFFER.clear();

    error!("error");

    let _: serde_json::Value = serde_json::from_slice(&common::OUTPUT_BUFFER.buffer())
        .expect("output doesn't parse as JSON");

    Ok(())
}

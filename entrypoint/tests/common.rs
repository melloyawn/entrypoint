#![allow(dead_code)]

use entrypoint::prelude::*;

#[derive(entrypoint::clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {}

impl LoggerConfig for Args {
    // pull level from env::var
    fn default_log_level(&self) -> entrypoint::tracing_subscriber::filter::LevelFilter {
        <entrypoint::tracing::Level as std::str::FromStr>::from_str(
            std::env::var("LOG_LEVEL")
                .unwrap_or(String::from("info"))
                .as_str(),
        )
        .unwrap()
        .into()
    }

    fn default_log_writer(&self) -> impl for<'writer> MakeWriter<'writer> + Send + Sync + 'static {
        std::io::sink
    }
}

////////////////////////////////////////////////////////////////////////////////
pub(crate) fn using_prod_env() -> entrypoint::anyhow::Result<()> {
    assert_eq!(std::env::var("APP_ENV")?, String::from("production"));
    assert_eq!(std::env::var("LOG_LEVEL")?, String::from("WARN"));
    assert_eq!(std::env::var("SECRET_KEY")?, String::from("BUT_NOT_REALLY"));
    assert!(std::env::var("TEST_KEY").is_err());
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
pub(crate) fn using_dev_env() -> entrypoint::anyhow::Result<()> {
    assert_eq!(std::env::var("APP_ENV")?, String::from("development"));
    assert_eq!(std::env::var("LOG_LEVEL")?, String::from("DEBUG"));
    assert_eq!(std::env::var("TEST_KEY")?, String::from("NOT_A_SECRET_KEY"));
    assert!(std::env::var("SECRET_KEY").is_err());
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
pub(crate) fn using_both_no_override() -> entrypoint::anyhow::Result<()> {
    assert_eq!(std::env::var("APP_ENV")?, String::from("production"));
    assert_eq!(std::env::var("LOG_LEVEL")?, String::from("WARN"));
    assert_eq!(std::env::var("TEST_KEY")?, String::from("NOT_A_SECRET_KEY"));
    assert_eq!(std::env::var("SECRET_KEY")?, String::from("BUT_NOT_REALLY"));
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
pub(crate) fn using_both_yes_override() -> entrypoint::anyhow::Result<()> {
    assert_eq!(std::env::var("APP_ENV")?, String::from("development"));
    assert_eq!(std::env::var("LOG_LEVEL")?, String::from("DEBUG"));
    assert_eq!(std::env::var("TEST_KEY")?, String::from("NOT_A_SECRET_KEY"));
    assert_eq!(std::env::var("SECRET_KEY")?, String::from("BUT_NOT_REALLY"));
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
pub(crate) fn verify_log_level<T: Logger>(
    args: &T,
    level: &tracing_subscriber::filter::LevelFilter,
) -> entrypoint::anyhow::Result<()> {
    // not the best test: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.LevelFilter.html#method.current
    assert!(*level <= entrypoint::tracing_subscriber::filter::LevelFilter::current());

    Ok(())
}

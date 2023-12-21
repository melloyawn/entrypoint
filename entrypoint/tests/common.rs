use entrypoint::prelude::*;

#[derive(entrypoint::clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {}

impl Logger for Args {
    fn log_level(&self) -> entrypoint::tracing::Level {
        <entrypoint::tracing::Level as std::str::FromStr>::from_str(
            std::env::var("LOG_LEVEL")
                .unwrap_or(String::from("info"))
                .as_str(),
        )
        .unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////
pub fn using_prod_env() -> entrypoint::anyhow::Result<()> {
    assert_eq!(std::env::var("APP_ENV")?, String::from("production"));
    assert_eq!(std::env::var("LOG_LEVEL")?, String::from("WARN"));
    assert_eq!(std::env::var("SECRET_KEY")?, String::from("BUT_NOT_REALLY"));
    assert!(std::env::var("TEST_KEY").is_err());
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
pub fn using_dev_env() -> entrypoint::anyhow::Result<()> {
    assert_eq!(std::env::var("APP_ENV")?, String::from("development"));
    assert_eq!(std::env::var("LOG_LEVEL")?, String::from("DEBUG"));
    assert_eq!(std::env::var("TEST_KEY")?, String::from("NOT_A_SECRET_KEY"));
    assert!(std::env::var("SECRET_KEY").is_err());
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
pub fn using_both_no_override() -> entrypoint::anyhow::Result<()> {
    assert_eq!(std::env::var("APP_ENV")?, String::from("production"));
    assert_eq!(std::env::var("LOG_LEVEL")?, String::from("WARN"));
    assert_eq!(std::env::var("TEST_KEY")?, String::from("NOT_A_SECRET_KEY"));
    assert_eq!(std::env::var("SECRET_KEY")?, String::from("BUT_NOT_REALLY"));
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
pub fn using_both_yes_override() -> entrypoint::anyhow::Result<()> {
    assert_eq!(std::env::var("APP_ENV")?, String::from("development"));
    assert_eq!(std::env::var("LOG_LEVEL")?, String::from("DEBUG"));
    assert_eq!(std::env::var("TEST_KEY")?, String::from("NOT_A_SECRET_KEY"));
    assert_eq!(std::env::var("SECRET_KEY")?, String::from("BUT_NOT_REALLY"));
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
pub fn verify_log_level<T: Logger>(
    args: &T,
    level: &entrypoint::tracing::Level,
) -> entrypoint::anyhow::Result<()> {
    assert_eq!(args.log_level(), *level);
    // not the best test: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.LevelFilter.html#method.current
    assert!(
        args.log_level()
            <= entrypoint::tracing_subscriber::filter::LevelFilter::current()
                .into_level()
                .unwrap()
    );
    Ok(())
}

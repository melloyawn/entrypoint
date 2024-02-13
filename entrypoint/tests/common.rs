#![allow(dead_code)]

use entrypoint::prelude::*;
use std::sync::{Arc, Mutex};

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
#[derive(Clone)]
pub struct BufferWriter {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl BufferWriter {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::<u8>::with_capacity(10))),
        }
    }

    pub fn buffer(&self) -> Vec<u8> {
        self.buffer.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.buffer.lock().unwrap().clear()
    }
}

impl std::io::Write for BufferWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

lazy_static::lazy_static! {
   pub static ref OUTPUT_BUFFER: BufferWriter = BufferWriter::new();
}

pub fn global_writer() -> BufferWriter {
    OUTPUT_BUFFER.clone()
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

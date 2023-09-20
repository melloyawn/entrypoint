#![forbid(unsafe_code)]

pub use anyhow;
pub use clap;
pub use tracing;

pub trait Entrypoint: clap::Parser {}

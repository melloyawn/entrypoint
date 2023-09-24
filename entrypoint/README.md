# entrypoint
[![Crates.io](https://img.shields.io/crates/v/entrypoint.svg)](https://crates.io/crates/entrypoint)
[![Crates.io](https://img.shields.io/crates/d/entrypoint.svg)](https://crates.io/crates/entrypoint)
[![Documentation](https://img.shields.io/docsrs/entrypoint?logo=docs.rs)](https://docs.rs/entrypoint)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE-MIT)

eliminate main function boilerplate with this opinionated application framework/wrapper

## About
`entrypoint` has the following design goals:
- eliminate application startup/configuration boilerplate
- help enforce application best practices

This crate integrates:
- [`anyhow`](https://github.com/dtolnay/anyhow) for easy application level error handling
- [`clap`](https://github.com/clap-rs/clap) for easy command-line argument parsing
- [`dotenvy`](https://github.com/allan2/dotenvy) for easy environment variable configuration

### What does this crate actually do?
// #FIXME

## Components
- [`entrypoint`](https://crates.io/crates/entrypoint):
- [`entrypoint_macros`](https://crates.io/crates/entrypoint_macros):

## Usage
1. Include the `entrypoint` prelude:
    ```rust
    use entrypoint::prelude::*;
    ```

2. Define a [`clap`](https://crates.io/crates/clap) struct:
    ```rust
    #[derive(entrypoint::clap::Parser, Debug)]
    #[command(version, about, long_about = None)]
    struct Args {
        #[arg(short, long)]
        debug: bool,
    }
    ```

3. Define a main/entrypoint function:
    ```rust
    #[entrypoint::entrypoint]
    fn entrypoint(args: Args) -> entrypoint::anyhow::Result<()> {
        entrypoint::tracing::info!("in entrypoint({:?})", args);
        Ok(())
    }
    ```
   This function must:
   1. accept a `clap::Parser` as an input
   2. return `entrypoint::anyhow::Result<()>`

**Note:** `#[entrypoint::entrypoint]` should be first when using with other attribute macros. e.g.:
    ```rust
    #[entrypoint::entrypoint]
    #[tokio::main]
    fn entrypoint(args: Args) -> entrypoint::anyhow::Result<()> { Ok(()) }
    ```

For more information:
- [docs.rs](https://docs.rs/entrypoint)
- [examples](examples/)

## Contributing
Before doing anything else: **open an issue**.

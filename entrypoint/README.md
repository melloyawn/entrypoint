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

### What does this crate actually do?
`entrypoint` essentially wraps a user defined function with automatic configuration/setup/processing of:
- logging (via [`tracing`](https://github.com/tokio-rs/tracing))
- command-line arguments (via [`clap`](https://github.com/clap-rs/clap))
- `.dotenv` file(s), environment variable population/overrides (via [`dotenvy`](https://github.com/allan2/dotenvy))
- easy application error handling (via [`anyhow`](https://github.com/dtolnay/anyhow))

The user defined function is intended to be `main`.

Thus, the main/entrypoint function can be written as if all the configuration/processing/boilerplate is ready-to-use.
More explicitly:
- `tracing` has been configured & the global subscriber has been registered
- `clap::Parser` struct has been parsed & populated
- `.dotenv` files have been parsed; environment variables are ready to go

Using the default behavior is totally reasonable, but [overwriting some traits' default impl(s)](tests/customize.rs) can provide customization.

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

3. Define an entrypoint/main function:
    ```rust
    #[entrypoint::entrypoint]
    fn main(args: Args) -> entrypoint::anyhow::Result<()> {
        // env::vars() already loaded-from/merged-with .dotenv file(s)
        let _dev_secret  = env::vars("SECRET_KEY");

        // logging is ready to use
        entrypoint::tracing::info!("entrypoint::entrypoint");

        // args is already parsed and ready to use
        entrypoint::tracing::info!("w/ {:#?}", args);

        Ok(())
    }
    ```
   This function must:
   1. accept a `clap::Parser` as an input
   2. return `entrypoint::anyhow::Result<()>`

**Note:** `#[entrypoint::entrypoint]` should be first when using with other attribute macros.
e.g.:
```rust
#[entrypoint::entrypoint]
#[tokio::main]
async fn main(args: Args) -> entrypoint::anyhow::Result<()> { Ok(()) }
```

For more information, refer to:
- [docs.rs](https://docs.rs/entrypoint)
- [tests](tests/)

## Crates
`entrypoint` is divided into the following crates:
- [`entrypoint`](https://crates.io/crates/entrypoint): core traits & functionality
- [`entrypoint_macros`](https://crates.io/crates/entrypoint_macros): convienence macros to further reduce boilerplate

## Contributing
Before doing anything else: **open an issue**.

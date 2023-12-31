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
`entrypoint` wraps a user defined function with automatic configuration/setup/processing of:
- easy application error handling (via [`anyhow`](https://github.com/dtolnay/anyhow))
- command-line argument parsing (via [`clap`](https://github.com/clap-rs/clap))
- `.dotenv` file processing and environment variable population/overrides (via [`dotenvy`](https://github.com/allan2/dotenvy))
- logging (via [`tracing`](https://github.com/tokio-rs/tracing))

The user defined function is intended to be/replace `main()`.

Meaning, this main/entrypoint function can be written as if all the configuration/processing/boilerplate is ready-to-use.
More explicitly:
- `anyhow` is available and ready to use
- `clap::Parser` struct has been parsed and populated
- `.dotenv` files have been parsed; environment variables are ready to go
- `tracing` has been configured and the global subscriber has been registered

## Usage
### Default Config
1. Include the `entrypoint` prelude:
    ```rust
    use entrypoint::prelude::*;
    ```

2. Define a [`clap`](https://crates.io/crates/clap) struct and [derive](/entrypoint_macros) default entrypoint trait impls:
    ```rust
    #[derive(clap::Parser, DotEnvDefault, LoggerDefault, Debug)]
    #[log_level(entrypoint::tracing::Level::DEBUG)]
    #[command(version, about, long_about = None)]
    struct CLIArgs {
        #[arg(short, long, env)]
        verbose: bool,
    }
    ```

3. Define an entrypoint/main function:
    ```rust
    #[entrypoint::entrypoint]
    fn entrypoint(args: CLIArgs) -> entrypoint::anyhow::Result<()> {
        // args are parsed and ready to use
        info!("verbose set to: {:?}", args.verbose);

        // env::vars() already loaded-from/merged-with .dotenv file(s)
        let _my_var = env::vars("SOMETHING_FROM_DOTENV_FILE");

        // logging is ready to use
        info!("entrypoint::entrypoint");

        Ok(())
    }
    ```

### Custom Config
Using the default behavior is totally reasonable, but [overwriting some default impl(s)](/entrypoint/examples/advanced_cli_args.rs) can provide customization.

### Usage Notes
1. The `entrypoint` function must:
   1. Have a `clap::Parser` input parameter
   2. return `entrypoint::anyhow::Result<()>`
2. `#[entrypoint::entrypoint]` ordering may matter when used with other attribute macros (e.g. `[tokio::main]`).

## Documentation
For more information, refer to:
- [docs.rs](https://docs.rs/entrypoint)
- [tera-former](https://github.com/melloyawn/tera-former)
- [examples](/entrypoint/examples/)
- [tests](/entrypoint/tests/)

## Crates
`entrypoint` is divided into the following crates:
- [`entrypoint`](https://crates.io/crates/entrypoint): core traits and functionality
- [`entrypoint_macros`](https://crates.io/crates/entrypoint_macros): convienence macros to further reduce boilerplate

## Contributing
Before doing anything else: **open an issue**.

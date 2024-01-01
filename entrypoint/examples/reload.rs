//! dynamic logging reload tui

use entrypoint::prelude::*;
use tokio::signal;

#[derive(entrypoint::clap::Parser, DotEnvDefault, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}

impl Logger for Args {
    fn log_layers<S>(
        &self,
    ) -> Option<Vec<Box<dyn tracing_subscriber::Layer<S> + Send + Sync + 'static>>>
    where
        S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    {
        let format = Format::default(); //#FIXME
        let layer = tracing_subscriber::fmt::layer()
            .event_format(format)
            .with_filter(self.log_level());
        let (layer, reload_handle) = reload::Layer::new(layer);

        Some(vec![layer.boxed()])
    }
}

#[tokio::main]
#[entrypoint::entrypoint]
async fn entrypoint(_args: Args) -> entrypoint::anyhow::Result<()> {
    let tui = tokio::spawn(async {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            trace!("this is a trace");
            debug!("this is a debug");
            info!("this is an info");
            warn!("this is a warn");
            error!("this is an error");
        }
    });

    match signal::ctrl_c().await {
        Ok(()) => {
            info!("recv'd ctrl-c; application shutdown");
        }
        Err(_) => {
            error!("failed to listen for ctrl-c");
        }
    }

    Ok(())
}

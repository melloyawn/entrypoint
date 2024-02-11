//! dynamic logging reload tui

use entrypoint::prelude::*;
use std::io;
use tokio::signal;

#[derive(entrypoint::clap::Parser, DotEnvDefault, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}

impl LoggerConfig for Args {
    fn default_log_layer(
        &self,
    ) -> Box<dyn tracing_subscriber::Layer<Registry> + Send + Sync + 'static> {
        let (layer, reload) = reload::Layer::new(
            tracing_subscriber::fmt::Layer::default()
                .event_format(self.default_log_format())
                .with_writer(self.default_log_writer())
                .with_filter(self.default_log_level()),
        );

        let _ = reload.modify(|layer| *layer.filter_mut() = self.default_log_level());
        //#FIXME let _ = reload.modify(|layer| {layer.inner_mut().map_event_format(|e| e);});
        let _ = reload.modify(|layer| *layer.inner_mut().writer_mut() = self.default_log_writer());

        layer.boxed()
    }
}

#[tokio::main]
#[entrypoint::entrypoint]
async fn entrypoint(_args: Args) -> entrypoint::anyhow::Result<()> {
    let logging = tokio::spawn(async {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            trace!("this is a trace");
            debug!("this is a debug");
            info!("this is an info");
            warn!("this is a warn");
            error!("this is an error");
        }
    });

    let cli = tokio::spawn(async {
        loop {
            let mut input = String::new();
            if let Ok(bytes) = io::stdin().read_line(&mut input) {
                error!(input);
            }
        }
    });

    let random_changes = tokio::spawn(async {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            //let _ = reload.modify(|layer| *layer.filter_mut() = self.default_log_level());
            //let _ = reload.modify(|layer| *layer.inner_mut().writer_mut() = self.default_log_writer());
            // #FIXME - format
            // #FIXME - thread name
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

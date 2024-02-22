//! use bypass_log_init to keep reload handle(s)

use entrypoint::prelude::*;

mod common;

#[derive(entrypoint::clap::Parser, DotEnvDefault, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}

impl entrypoint::LoggerConfig for Args {
    fn bypass_log_init(&self) -> bool {
        true
    }
    fn default_log_writer(&self) -> impl for<'writer> MakeWriter<'writer> + Send + Sync + 'static {
        std::io::sink
    }
}

#[entrypoint::entrypoint]
#[test]
fn entrypoint(args: Args) -> entrypoint::anyhow::Result<()> {
    // manually config/init logging
    let (layer_one, reload_one) = reload::Layer::new(
        tracing_subscriber::fmt::Layer::default()
            .event_format(args.default_log_format())
            .with_writer(args.default_log_writer())
            .with_filter(args.default_log_level()),
    );
    let (layer_two, reload_two) = reload::Layer::new(
        tracing_subscriber::fmt::Layer::default()
            .event_format(args.default_log_format())
            .with_writer(args.default_log_writer())
            .with_filter(args.default_log_level()),
    );
    let _args = args.log_init(Some(vec![layer_one.boxed(), layer_two.boxed()]))?;

    ////////////////////////////////////////////////////////////////////////////
    // independent control/reload of filter
    let _ = reload_one.modify(|layer| *layer.filter_mut() = entrypoint::LevelFilter::INFO);
    let _ = reload_two.modify(|layer| *layer.filter_mut() = entrypoint::LevelFilter::INFO);
    assert!(!enabled!(entrypoint::Level::TRACE));
    assert!(enabled!(entrypoint::Level::INFO));

    let _ = reload_one.modify(|layer| *layer.filter_mut() = entrypoint::LevelFilter::TRACE);
    let _ = reload_two.modify(|layer| *layer.filter_mut() = entrypoint::LevelFilter::INFO);
    assert!(enabled!(entrypoint::Level::TRACE));

    let _ = reload_one.modify(|layer| *layer.filter_mut() = entrypoint::LevelFilter::INFO);
    let _ = reload_two.modify(|layer| *layer.filter_mut() = entrypoint::LevelFilter::TRACE);
    assert!(enabled!(entrypoint::Level::TRACE));

    let _ = reload_one.modify(|layer| *layer.filter_mut() = entrypoint::LevelFilter::INFO);
    let _ = reload_two.modify(|layer| *layer.filter_mut() = entrypoint::LevelFilter::INFO);
    assert!(!enabled!(entrypoint::Level::TRACE));
    assert!(enabled!(entrypoint::Level::INFO));

    ////////////////////////////////////////////////////////////////////////////
    // independent control/reload of writer & format
    assert!(serde_json::from_slice::<serde_json::Value>(&common::OUTPUT_BUFFER.buffer()).is_err());

    // #FIXME - waiting on https://github.com/tokio-rs/tracing/pull/1959
    //let _ = reload_one.modify(|layer| {
    //    layer.inner_mut().map_event_format(|e| e);
    //});
    //#FIXME let _ = reload_one.modify(|layer| *layer.inner_mut().writer_mut() = common::global_writer);
    //#FIXME let _ = reload_one.modify(|layer| *layer.inner_mut().writer_mut() = std::io::stdout);

    common::OUTPUT_BUFFER.clear();
    error!("error");
    //#FIXME assert!(serde_json::from_slice::<serde_json::Value>(&common::OUTPUT_BUFFER.buffer()).is_ok());

    Ok(())
}

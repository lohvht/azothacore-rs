use tracing::{subscriber::set_global_default, Level};
use tracing_subscriber::{filter::filter_fn, prelude::*, Registry};

/// Compose multiple layers into a `tracing`'s subscriber.
///
/// Then register the subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let (fw, fwguard) = tracing_appender::non_blocking(tracing_appender::rolling::never("logs", "log.txt"));

    let subscriber = Registry::default()
        .with(
            tracing_subscriber::fmt::Layer::default()
                .with_writer(fw)
                .with_ansi(false)
                .with_filter(filter_fn(|metadata| metadata.level().cmp(&Level::DEBUG).is_lt())),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(true)
                .with_filter(filter_fn(|metadata| metadata.level().cmp(&Level::DEBUG).is_lt())),
        )
        .with(console_subscriber::spawn());
    set_global_default(subscriber).expect("Failed to set subscriber");

    fwguard
}

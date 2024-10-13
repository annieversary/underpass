use std::collections::HashMap;

use backtrace::Backtrace;
use opentelemetry::trace::TracerProvider as _;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn setup_tracing() -> WorkerGuard {
    let log_path = std::env::var("LOG_PATH").expect("failed to get LOG_PATH");

    // normal logging
    let registry = tracing_subscriber::registry().with(EnvFilter::from_env("LOG_LEVEL"));

    let (fmt_layer, guard1) = {
        // file
        let file_appender = tracing_appender::rolling::daily(log_path, "app.log");
        #[allow(unused_variables)]
        let (non_blocking, guard1) = tracing_appender::non_blocking(file_appender);

        let fmt_layer = tracing_subscriber::fmt::layer()
            .pretty()
            .with_writer(non_blocking);

        // on debug, also log to stdout
        #[cfg(debug_assertions)]
        let fmt_layer = fmt_layer.with_writer(std::io::stdout);

        (fmt_layer, guard1)
    };

    let enable_otel = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok();
    let telemetry_layer = if enable_otel {
        let telemetry_exporter = opentelemetry_otlp::new_exporter()
            .http()
            .with_headers(parse_otel_headers());

        let telemetry_tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(telemetry_exporter)
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .expect("Couldn't create OTLP tracer")
            .tracer(env!("CARGO_PKG_NAME"));

        // let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        let telemetry_layer = OpenTelemetryLayer::new(telemetry_tracer);

        Some(telemetry_layer)
    } else {
        None
    };

    registry.with(fmt_layer).with(telemetry_layer).init();

    set_panic_hook();

    guard1
}

fn parse_otel_headers() -> HashMap<String, String> {
    let otel_headers: String = std::env::var("OTEL_EXPORTER_OTLP_HEADERS")
        .expect("failed to get OTEL_EXPORTER_OTLP_HEADERS");

    otel_headers
        .split(',')
        .flat_map(|header| header.split_once('='))
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect()
}

fn set_panic_hook() {
    std::panic::set_hook(Box::new(|panic| {
        let b = Backtrace::new();
        if let Some(location) = panic.location() {
            tracing::error!(
                message = %panic,
                panic.file = location.file(),
                panic.line = location.line(),
                panic.column = location.column(),
                backtrace = ?b,
            );
        } else {
            tracing::error!(message = %panic, backtrace = ?b);
        }
    }));
}

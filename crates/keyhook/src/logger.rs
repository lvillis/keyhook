//! Initialise a *console-only* `tracing` subscriber (no file output).

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Install the global subscriber.
///
/// * Re-invocation is silently ignored.
/// * Log level is read from `KEYHOOK_LOG` (`info` by default).
pub fn init() {
    if tracing::dispatcher::has_been_set() {
        // Prevent double initialisation
        return;
    }

    // `RUST_LOG`-style filter, defaulting to informative output
    let filter = tracing_subscriber::EnvFilter::try_from_env("KEYHOOK_LOG")
        .unwrap_or_else(|_| "info,keyhook=debug,tauri_runtime_wry=warn".into());

    // Colourised output, UTC timestamp, line numbers, thread IDs
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339());

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}

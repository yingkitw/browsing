//! Logging configuration for browser-use-rs

use tracing_subscriber::fmt;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize logging for browser-use-rs
pub fn setup_logging() {
    // Get log level from environment or default to INFO
    let log_level = std::env::var("BROWSER_USE_LOGGING_LEVEL")
        .unwrap_or_else(|_| "info".to_string())
        .to_lowercase();

    let filter = match log_level.as_str() {
        "trace" => EnvFilter::new("trace"),
        "debug" => EnvFilter::new("debug"),
        "info" => EnvFilter::new("info"),
        "warn" => EnvFilter::new("warn"),
        "error" => EnvFilter::new("error"),
        _ => EnvFilter::new("info"),
    };

    Registry::default()
        .with(filter)
        .with(fmt::layer().with_target(false))
        .init();
}

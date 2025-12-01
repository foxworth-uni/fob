//! Logging utilities for fob-bundler
//!
//! This module is only available with the `logging` feature.
//!
//! For library users: fob emits tracing events - install your own subscriber.
//! For application developers: use these convenience functions.

use std::sync::Once;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

static INIT: Once = Once::new();

/// Log level for fob output
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LogLevel {
    /// No logging output
    Silent,
    /// Only errors
    Error,
    /// Errors and warnings
    Warn,
    /// Errors, warnings, and info (default)
    #[default]
    Info,
    /// All logs including debug
    Debug,
}

impl LogLevel {
    /// Convert to tracing filter string
    fn as_filter(&self) -> &'static str {
        match self {
            LogLevel::Silent => "off",
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "silent" | "off" => Ok(LogLevel::Silent),
            "error" => Ok(LogLevel::Error),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            other => Err(format!("Invalid log level: {}", other)),
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_filter())
    }
}

/// Initialize fob logging with specified level
///
/// # Thread Safety
///
/// This function installs a global subscriber and should only be called once
/// per process. It is safe to call from multiple threads - only the first
/// call will take effect.
///
/// # Example
///
/// ```rust,no_run
/// use fob_bundler::logging::{init_logging, LogLevel};
///
/// init_logging(LogLevel::Info);
/// ```
pub fn init_logging(level: LogLevel) {
    INIT.call_once(|| {
        let filter = EnvFilter::builder()
            .with_default_directive(level.as_filter().parse().unwrap())
            .from_env_lossy();

        tracing_subscriber::registry()
            .with(filter)
            .with(
                fmt::layer().compact().with_target(false).without_time(), // Let consumers control timestamp format
            )
            .init();
    });
}

/// Initialize logging from RUST_LOG environment variable
///
/// Falls back to Info level if RUST_LOG is not set or invalid.
///
/// # Example
///
/// ```rust,no_run
/// use fob_bundler::logging::init_logging_from_env;
///
/// init_logging_from_env();
/// ```
pub fn init_logging_from_env() {
    INIT.call_once(|| {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::builder()
                .with_default_directive("info".parse().unwrap())
                .from_env_lossy()
        });

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().compact().with_target(false).without_time())
            .init();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_str() {
        assert_eq!("info".parse::<LogLevel>().unwrap(), LogLevel::Info);
        assert_eq!("debug".parse::<LogLevel>().unwrap(), LogLevel::Debug);
        assert_eq!("warn".parse::<LogLevel>().unwrap(), LogLevel::Warn);
        assert_eq!("warning".parse::<LogLevel>().unwrap(), LogLevel::Warn);
        assert_eq!("error".parse::<LogLevel>().unwrap(), LogLevel::Error);
        assert_eq!("silent".parse::<LogLevel>().unwrap(), LogLevel::Silent);
        assert_eq!("off".parse::<LogLevel>().unwrap(), LogLevel::Silent);
        assert_eq!("INFO".parse::<LogLevel>().unwrap(), LogLevel::Info);
        assert!("invalid".parse::<LogLevel>().is_err());
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Info.to_string(), "info");
        assert_eq!(LogLevel::Debug.to_string(), "debug");
        assert_eq!(LogLevel::Silent.to_string(), "off");
    }

    #[test]
    fn test_log_level_default() {
        assert_eq!(LogLevel::default(), LogLevel::Info);
    }
}

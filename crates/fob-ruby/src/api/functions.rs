//! Standalone Ruby functions

use crate::types::LogLevel;
use magnus::{Error, Ruby};

/// Initialize fob logging with specified level
///
/// Call this once at application startup before any fob operations.
pub fn init_logging(_ruby: &Ruby, level: Option<String>) -> Result<(), Error> {
    let log_level = level
        .as_ref()
        .and_then(|s| LogLevel::from_str(s))
        .unwrap_or_default();
    fob_bundler::init_logging(log_level.into());
    Ok(())
}

/// Initialize logging from RUST_LOG environment variable
///
/// Falls back to Info level if RUST_LOG is not set or invalid.
pub fn init_logging_from_env(_ruby: &Ruby) -> Result<(), Error> {
    fob_bundler::init_logging_from_env();
    Ok(())
}

/// Get the bundler version
pub fn version(_ruby: &Ruby) -> Result<String, Error> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

//! Type definitions for fob-native

use napi_derive::napi;

/// Output format for bundled code
#[napi(string_enum)]
#[derive(Clone, Debug)]
pub enum OutputFormat {
    /// ES Module format
    Esm,
    /// CommonJS format
    Cjs,
    /// Immediately Invoked Function Expression format
    Iife,
}

/// Log level for fob output
///
/// Controls the verbosity of logging during bundling operations.
#[napi(string_enum)]
#[derive(Clone, Debug, Default)]
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

impl From<LogLevel> for fob_bundler::LogLevel {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Silent => fob_bundler::LogLevel::Silent,
            LogLevel::Error => fob_bundler::LogLevel::Error,
            LogLevel::Warn => fob_bundler::LogLevel::Warn,
            LogLevel::Info => fob_bundler::LogLevel::Info,
            LogLevel::Debug => fob_bundler::LogLevel::Debug,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_default() {
        assert!(matches!(LogLevel::default(), LogLevel::Info));
    }

    #[test]
    fn test_log_level_to_bundler() {
        // Test all variants convert correctly
        assert!(matches!(
            fob_bundler::LogLevel::from(LogLevel::Silent),
            fob_bundler::LogLevel::Silent
        ));
        assert!(matches!(
            fob_bundler::LogLevel::from(LogLevel::Error),
            fob_bundler::LogLevel::Error
        ));
        assert!(matches!(
            fob_bundler::LogLevel::from(LogLevel::Warn),
            fob_bundler::LogLevel::Warn
        ));
        assert!(matches!(
            fob_bundler::LogLevel::from(LogLevel::Info),
            fob_bundler::LogLevel::Info
        ));
        assert!(matches!(
            fob_bundler::LogLevel::from(LogLevel::Debug),
            fob_bundler::LogLevel::Debug
        ));
    }
}

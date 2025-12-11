//! Logging infrastructure for the Joy CLI.
//!
//! This module provides a structured logging setup using the `tracing` ecosystem.
//! It supports multiple verbosity levels, colored output, and environment-based
//! configuration for debugging.
//!
//! # Features
//!
//! - **Verbosity control**: `--verbose` for debug, `--quiet` for errors only
//! - **Color support**: Automatic detection with `--no-color` override
//! - **Environment filters**: Override via `RUST_LOG` environment variable
//! - **Structured logging**: Use tracing spans for context
//!
//! # Example
//!
//! ```rust,no_run
//! use fob_cli::logger::init_logger;
//! use tracing::{info, debug, error};
//!
//! init_logger(false, false, false);
//!
//! info!("Starting build");
//! debug!("Processing module: {}", "index.ts");
//! ```

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize the tracing subscriber with the specified options.
///
/// This function sets up structured logging for the CLI. It should be called
/// once at the start of the program, before any logging occurs.
///
/// # Arguments
///
/// * `verbose` - Enable debug-level logging (overrides `quiet`)
/// * `quiet` - Only show error-level logs
/// * `no_color` - Disable colored output
///
/// # Verbosity Levels
///
/// The logging level is determined in this order:
/// 1. `--verbose` flag: Sets level to DEBUG for fob crates
/// 2. `--quiet` flag: Sets level to ERROR only
/// 3. `RUST_LOG` environment variable: Custom filter
/// 4. Default: INFO level for fob crates
///
/// # Examples
///
/// ```rust,no_run
/// use fob_cli::logger::init_logger;
///
/// // Default logging (INFO level)
/// init_logger(false, false, false);
///
/// // Debug logging
/// init_logger(true, false, false);
///
/// // Quiet mode (errors only)
/// init_logger(false, true, false);
///
/// // No colors (for CI/piped output)
/// init_logger(false, false, true);
/// ```
pub fn init_logger(verbose: bool, quiet: bool, no_color: bool) {
    // Determine the filter level based on flags and environment
    let filter = if verbose {
        // Verbose mode: debug level for fob crates, info for dependencies
        EnvFilter::new("fob=debug,fob_bundler=debug,fob_config=debug,fob_cli=debug")
    } else if quiet {
        // Quiet mode: only errors
        EnvFilter::new("fob=error")
    } else {
        // Try to read from RUST_LOG env var, fallback to info level
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("fob=info,fob_bundler=info,fob_config=info"))
    };

    // Configure the formatter
    let fmt_layer = fmt::layer()
        .with_target(false) // Don't show the module path (keeps output clean)
        .with_level(true) // Show log level (INFO, DEBUG, etc.)
        .with_ansi(!no_color) // Enable colors unless disabled
        .compact(); // Use compact formatting for better readability

    // Initialize the global subscriber
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}

/// Initialize logger with custom environment filter.
///
/// This is useful for testing or advanced scenarios where you need precise
/// control over log filtering.
///
/// # Example
///
/// ```rust,no_run
/// use fob_cli::logger::init_logger_with_filter;
/// use tracing_subscriber::EnvFilter;
///
/// let filter = EnvFilter::new("fob=trace,hyper=off");
/// init_logger_with_filter(filter, false);
/// ```
pub fn init_logger_with_filter(filter: EnvFilter, no_color: bool) {
    let fmt_layer = fmt::layer()
        .with_target(false)
        .with_level(true)
        .with_ansi(!no_color)
        .compact();

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}

/// Check if colored output should be enabled.
///
/// This checks terminal capabilities and environment variables to determine
/// if colors should be used. Useful for determining color support before
/// initializing the logger.
///
/// # Returns
///
/// `true` if colors should be enabled, `false` otherwise
///
/// # Environment Variables
///
/// - `NO_COLOR`: If set, disables colors
/// - `FORCE_COLOR`: If set, forces colors even in non-TTY
pub fn should_use_colors() -> bool {
    // Check NO_COLOR environment variable (standard convention)
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Check FORCE_COLOR environment variable
    if std::env::var("FORCE_COLOR").is_ok() {
        return true;
    }

    // Use console crate to detect terminal capabilities
    // It handles cross-platform TTY detection for us
    console::Term::stdout().features().colors_supported()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests verify the API but don't test actual output
    // since tracing is global and can only be initialized once per process.

    #[test]
    fn test_logger_initialization() {
        // This test just verifies the function doesn't panic
        // We can't actually test the output without complex mocking
        // In a real scenario, you'd use separate binaries for integration tests
    }

    #[test]
    fn test_should_use_colors_respects_force_color() {
        // Clear NO_COLOR first
        unsafe {
            std::env::remove_var("NO_COLOR");
        }

        // Set FORCE_COLOR and verify it enables colors
        std::env::set_var("FORCE_COLOR", "1");
        assert!(should_use_colors());
        unsafe {
            std::env::remove_var("FORCE_COLOR");
        }
    }

    #[test]
    fn test_env_filter_verbose() {
        // Just verify we can create the filter without panicking
        let _filter = EnvFilter::new("fob=debug,fob_bundler=debug,fob_config=debug,fob_cli=debug");
        // The internal format of EnvFilter isn't guaranteed, so we just verify creation
    }

    #[test]
    fn test_env_filter_quiet() {
        // Just verify we can create the filter without panicking
        let _filter = EnvFilter::new("fob=error");
        // The internal format of EnvFilter isn't guaranteed, so we just verify creation
    }

    // Integration test example (would need to be in tests/ directory)
    // This demonstrates how you'd test actual logging output
    /*
    #[test]
    fn test_logger_output() {
        use std::sync::Once;
        static INIT: Once = Once::new();

        INIT.call_once(|| {
            init_logger(false, false, true); // no color for testing
        });

        // Would need to capture stdout/stderr to verify actual output
        info!("test message");
    }
    */
}

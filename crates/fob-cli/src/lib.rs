//! Fob CLI - Modern JavaScript bundler with Rust performance.
//!
//! This crate provides the command-line interface for the Fob bundler, exposing
//! all functionality from `fob-core` through an intuitive CLI with excellent
//! error messages and user experience.
//!
//! # Architecture
//!
//! The CLI is organized into several key modules:
//!
//! - [`error`] - Comprehensive error types with actionable messages
//! - [`logger`] - Structured logging with tracing
//! - [`ui`] - Terminal UI utilities for progress bars and formatted output
//! - `commands` - Individual CLI command implementations
//! - `config` - Configuration file handling
//! - `server` - Development server
//!
//! # Features
//!
//! - **Type-safe error handling**: Uses `thiserror` for structured errors
//! - **Structured logging**: Built on `tracing` for better debugging
//! - **Beautiful terminal UI**: Progress bars, colors, and formatting
//! - **File watching**: Automatic rebuilds on file changes
//! - **Configuration profiles**: Environment-specific settings
//!
//! # Example
//!
//! ```rust
//! use fob_cli::{error::Result, logger};
//!
//! fn main() -> Result<()> {
//!     logger::init_logger(false, false, false);
//!     // CLI command implementations...
//!     Ok(())
//! }
//! ```

// Public modules
pub mod cli;
pub mod commands;
pub mod config;
pub mod dev;
pub mod error;
pub mod logger;
pub mod ui;

// Re-export commonly used types
pub use error::{BuildError, CliError, ConfigError, Result, ResultExt};

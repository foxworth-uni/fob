//! Command implementations for the Joy CLI.
//!
//! This module contains the implementation of all CLI commands:
//!
//! - [`build`] - Bundle JavaScript/TypeScript files
//! - [`dev`] - Development server with hot reload
//! - [`init`] - Project scaffolding
//! - [`check`] - Configuration validation
//!
//! Each command is implemented in its own module and provides an `execute`
//! function that takes the parsed command arguments and returns a Result.

pub mod build;
pub mod check;
pub mod dev;
pub mod init;
mod templates;
pub(crate) mod utils;

// Re-export execute functions for convenience
pub use build::execute as build_execute;
pub use check::execute as check_execute;
pub use dev::execute as dev_execute;
pub use init::execute as init_execute;

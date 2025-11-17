//! Command-line interface definition for Fob bundler.
//!
//! This module defines the complete CLI structure using clap v4's derive macros.
//! It provides type-safe argument parsing with comprehensive validation and
//! clear error messages.
//!
//! # Command Structure
//!
//! - `fob build` - Bundle JavaScript/TypeScript with full configuration
//! - `fob dev` - Development server with watch mode (planned)
//! - `fob init` - Project scaffolding (planned)
//! - `fob check` - Configuration validation (planned)

mod commands;
pub mod enums;
mod tests;
mod validation;

use clap::Parser;

pub use commands::{BuildArgs, CheckArgs, Command, DevArgs, InitArgs};
pub use enums::*;
pub use validation::parse_global;

/// Fob - A modern JavaScript/TypeScript bundler
#[derive(Parser, Debug)]
#[command(
    name = "fob",
    version,
    about = "A modern JavaScript/TypeScript bundler",
    long_about = "Fob is a fast, modern bundler for JavaScript and TypeScript projects.\n\
                  It provides zero-config bundling with support for ESM, CJS, and IIFE formats,\n\
                  TypeScript declaration generation, and optimized production builds."
)]
pub struct Cli {
    /// Enable verbose logging (debug level)
    ///
    /// Shows detailed information about the bundling process, including
    /// file transformations, dependency resolution, and performance metrics.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all output except errors
    ///
    /// Only critical errors will be displayed. Useful for CI/CD environments
    /// or when piping output to other tools.
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Disable colored output
    ///
    /// Outputs plain text without ANSI color codes. Useful for logging to
    /// files or systems that don't support colored terminal output.
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Command,
}


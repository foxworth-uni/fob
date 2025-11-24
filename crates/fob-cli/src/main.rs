//! Joy CLI - Modern JavaScript bundler with Rust performance.
//!
//! This is the main entry point for the Joy CLI. It handles command-line argument
//! parsing, logging initialization, and command dispatch.

use clap::Parser;
use fob_cli::{cli, commands, error, logger, ui};
use miette::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = cli::Cli::parse();

    // Initialize logging and colors based on global flags
    logger::init_logger(args.verbose, args.quiet, args.no_color);
    ui::init_colors();

    // Execute the appropriate command
    let result = match args.command {
        cli::Command::Build(build_args) => commands::build_execute(build_args).await,
        cli::Command::Dev(dev_args) => commands::dev_execute(dev_args).await,
        cli::Command::Init(init_args) => commands::init_execute(init_args).await,
        cli::Command::Check(check_args) => commands::check_execute(check_args).await,
    };

    // Convert CLI errors to miette diagnostics for beautiful error reporting
    result.map_err(error::cli_error_to_miette)
}

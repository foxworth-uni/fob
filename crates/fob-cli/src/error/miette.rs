//! Miette diagnostic conversion for CLI errors.
//!
//! This module provides conversion from CLI errors to miette diagnostics
//! for beautiful error reporting.

use crate::error::{BuildError, CliError};
use fob_bundler::diagnostics::to_diagnostic_error;
use miette::Report;

/// Convert CliError to miette Report
pub fn cli_error_to_miette(err: CliError) -> Report {
    match err {
        CliError::Build(e) => build_error_to_miette(e),
        CliError::Config(e) => miette::miette!("Configuration error: {}", e),
        CliError::Core(msg) => {
            // Try to parse as bundler error
            miette::miette!("Bundler error: {}", msg)
        }
        _ => miette::miette!("{}", err),
    }
}

/// Convert BuildError to miette Report
pub fn build_error_to_miette(err: BuildError) -> Report {
    match err {
        BuildError::ResolutionFailed { module, importer, hint } => {
            miette::miette!(
                "Failed to resolve module: {}\nImported from: {}\n\nHint: {}",
                module,
                importer.display(),
                hint
            )
        }
        BuildError::CircularDependency { cycle } => {
            miette::miette!(
                "Circular dependency detected:\n{}\n\nHint: Refactor to remove circular imports",
                cycle
            )
        }
        BuildError::TransformError { file, error, hint } => {
            miette::miette!(
                "Transform error in {}: {}\n\nHint: {}",
                file.display(),
                error,
                hint
            )
        }
        _ => miette::miette!("{}", err),
    }
}

/// Convert fob-bundler Error to miette Report
pub fn bundler_error_to_miette(err: fob_bundler::Error) -> Report {
    match err {
        fob_bundler::Error::Bundler(diagnostics) => {
            if diagnostics.is_empty() {
                miette::miette!("Unknown bundler error")
            } else if diagnostics.len() == 1 {
                // Convert single diagnostic to miette
                let diag_error = to_diagnostic_error(diagnostics.into_iter().next().unwrap());
                miette::Report::new(diag_error)
            } else {
                // Multiple diagnostics - use the first one as primary
                let primary_diag = diagnostics.first().unwrap();
                let diag_error = to_diagnostic_error(primary_diag.clone());
                miette::Report::new(diag_error)
            }
        }
        _ => {
            // Use the Diagnostic implementation we added to Error
            miette::Report::new(err)
        }
    }
}


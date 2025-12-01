//! Error types for Tailwind CSS CLI integration

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during Tailwind CSS generation
#[derive(Error, Debug)]
pub enum GeneratorError {
    /// Tailwind CLI not found (no package.json or lockfile)
    #[error("Tailwind CLI not found. Searched in: {searched_paths:?}")]
    CliNotFound { searched_paths: Vec<PathBuf> },

    /// Package manager binary not found in PATH
    #[error("Package manager '{package_manager}' binary '{binary_name}' not found in PATH")]
    PackageManagerNotFound {
        package_manager: String,
        binary_name: String,
    },

    /// Failed to spawn the Tailwind CLI process
    #[error("Failed to spawn Tailwind CLI process: {source}")]
    SpawnFailed {
        #[source]
        source: std::io::Error,
    },

    /// CLI process exited with non-zero status
    #[error("Tailwind CLI exited with code {exit_code}: {stderr}")]
    CliExitError { exit_code: i32, stderr: String },

    /// Output from CLI exceeded maximum allowed size
    #[error("CLI output too large: {actual_bytes} bytes (max: {max_bytes} bytes)")]
    OutputTooLarge {
        actual_bytes: usize,
        max_bytes: usize,
    },

    /// Failed to parse CLI output as valid UTF-8
    #[error("Failed to parse CLI output as UTF-8: {source}")]
    ParseError {
        #[source]
        source: std::string::FromUtf8Error,
    },

    /// CLI process timed out
    #[error("Tailwind CLI timed out after {timeout_secs} seconds")]
    Timeout { timeout_secs: u64 },
}

impl GeneratorError {
    pub fn cli_not_found(searched_paths: Vec<PathBuf>) -> Self {
        Self::CliNotFound { searched_paths }
    }

    pub fn spawn_failed(source: std::io::Error) -> Self {
        Self::SpawnFailed { source }
    }

    pub fn cli_exit_error(exit_code: i32, stderr: String) -> Self {
        Self::CliExitError { exit_code, stderr }
    }

    pub fn output_too_large(actual_bytes: usize, max_bytes: usize) -> Self {
        Self::OutputTooLarge {
            actual_bytes,
            max_bytes,
        }
    }

    pub fn parse_error(source: std::string::FromUtf8Error) -> Self {
        Self::ParseError { source }
    }

    pub fn timeout(timeout_secs: u64) -> Self {
        Self::Timeout { timeout_secs }
    }
}

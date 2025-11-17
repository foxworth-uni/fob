//! Error types for Tailwind CSS CLI integration

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during Tailwind CSS generation
#[derive(Error, Debug)]
pub enum GeneratorError {
    /// Tailwind CLI binary not found in any search location
    #[error("Tailwind CLI not found. Searched in: {searched_paths:?}")]
    CliNotFound {
        /// Paths that were searched for the CLI
        searched_paths: Vec<PathBuf>,
    },

    /// CLI binary found but is not executable
    #[error("Tailwind CLI found at {path} but is not executable")]
    CliNotExecutable {
        /// Path to the non-executable CLI binary
        path: PathBuf,
    },

    /// Failed to spawn the Tailwind CLI process
    #[error("Failed to spawn Tailwind CLI process: {source}")]
    SpawnFailed {
        /// The underlying IO error
        #[source]
        source: std::io::Error,
    },

    /// CLI process exited with non-zero status
    #[error("Tailwind CLI exited with code {exit_code}: {stderr}")]
    CliExitError {
        /// Exit code from the CLI process
        exit_code: i32,
        /// Standard error output from the CLI
        stderr: String,
    },

    /// Invalid CSS class candidate (security validation failed)
    #[error("Invalid CSS candidate '{candidate}': {reason}")]
    InvalidCandidate {
        /// The candidate that failed validation
        candidate: String,
        /// Reason for validation failure
        reason: String,
    },

    /// Output from CLI exceeded maximum allowed size
    #[error("CLI output too large: {actual_bytes} bytes (max: {max_bytes} bytes)")]
    OutputTooLarge {
        /// Actual size of the output
        actual_bytes: usize,
        /// Maximum allowed size
        max_bytes: usize,
    },

    /// Failed to parse CLI output as valid UTF-8
    #[error("Failed to parse CLI output as UTF-8: {source}")]
    ParseError {
        /// The underlying UTF-8 error
        #[source]
        source: std::string::FromUtf8Error,
    },

    /// IO error during CLI interaction
    #[error("IO error during Tailwind CLI operation: {source}")]
    IoError {
        /// The underlying IO error
        #[from]
        source: std::io::Error,
    },

    /// CLI process timed out
    #[error("Tailwind CLI process timed out after {timeout_secs} seconds")]
    Timeout {
        /// Timeout duration in seconds
        timeout_secs: u64,
    },
}

impl GeneratorError {
    /// Create a CliNotFound error with search paths
    pub fn cli_not_found(searched_paths: Vec<PathBuf>) -> Self {
        Self::CliNotFound { searched_paths }
    }

    /// Create a CliNotExecutable error
    pub fn cli_not_executable(path: PathBuf) -> Self {
        Self::CliNotExecutable { path }
    }

    /// Create a SpawnFailed error
    pub fn spawn_failed(source: std::io::Error) -> Self {
        Self::SpawnFailed { source }
    }

    /// Create a CliExitError
    pub fn cli_exit_error(exit_code: i32, stderr: String) -> Self {
        Self::CliExitError { exit_code, stderr }
    }

    /// Create an InvalidCandidate error
    pub fn invalid_candidate(candidate: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidCandidate {
            candidate: candidate.into(),
            reason: reason.into(),
        }
    }

    /// Create an OutputTooLarge error
    pub fn output_too_large(actual_bytes: usize, max_bytes: usize) -> Self {
        Self::OutputTooLarge {
            actual_bytes,
            max_bytes,
        }
    }

    /// Create a ParseError
    pub fn parse_error(source: std::string::FromUtf8Error) -> Self {
        Self::ParseError { source }
    }

    /// Create a Timeout error
    pub fn timeout(timeout_secs: u64) -> Self {
        Self::Timeout { timeout_secs }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_not_found_error() {
        let paths = vec![PathBuf::from("/usr/bin"), PathBuf::from("/usr/local/bin")];
        let error = GeneratorError::cli_not_found(paths.clone());
        let msg = error.to_string();
        assert!(msg.contains("not found"));
        assert!(msg.contains("/usr/bin"));
    }

    #[test]
    fn test_invalid_candidate_error() {
        let error = GeneratorError::invalid_candidate("../etc/passwd", "path traversal");
        let msg = error.to_string();
        assert!(msg.contains("Invalid CSS candidate"));
        assert!(msg.contains("../etc/passwd"));
        assert!(msg.contains("path traversal"));
    }

    #[test]
    fn test_output_too_large_error() {
        let error = GeneratorError::output_too_large(100_000_000, 50_000_000);
        let msg = error.to_string();
        assert!(msg.contains("too large"));
        assert!(msg.contains("100000000"));
        assert!(msg.contains("50000000"));
    }

    #[test]
    fn test_timeout_error() {
        let error = GeneratorError::timeout(30);
        let msg = error.to_string();
        assert!(msg.contains("timed out"));
        assert!(msg.contains("30"));
    }
}

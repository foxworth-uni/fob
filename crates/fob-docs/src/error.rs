use std::path::PathBuf;

use thiserror::Error;

/// Result type alias for documentation operations.
pub type Result<T> = std::result::Result<T, DocsError>;

/// Error variants for documentation extraction and generation.
#[derive(Debug, Error)]
pub enum DocsError {
    /// Failed to read or access a source file.
    #[error("failed to read source '{path}': {error}")]
    Io {
        /// Path to the source file that caused the error.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        error: std::io::Error,
    },

    /// Parsing the source file with OXC failed.
    #[error("failed to parse source '{path}': {message}")]
    Parse {
        /// Path to the source file.
        path: PathBuf,
        /// Aggregated parser error message.
        message: String,
    },

    /// Attempted to extract documentation for an unsupported export shape.
    #[error("unsupported export in '{path}': {details}")]
    UnsupportedExport {
        /// Path to the source file.
        path: PathBuf,
        /// Additional context regarding the unsupported export.
        details: String,
    },

    /// Generic error variant.
    #[error("{message}")]
    Other {
        /// Human-readable error message.
        message: String,
    },
}

impl DocsError {
    /// Helper to create a parse error from multiple diagnostic strings.
    pub fn parse_error(path: PathBuf, diagnostics: &[String]) -> Self {
        let message = diagnostics.join("; ");
        Self::Parse { path, message }
    }
}

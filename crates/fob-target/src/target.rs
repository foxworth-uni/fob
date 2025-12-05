//! DeploymentTarget trait re-export and error types.

// Re-export the trait from fob-bundler
pub use fob_bundler::DeploymentTarget;

/// Result type for deployment target operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for deployment target operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Build result error: {0}")]
    Build(String),
}

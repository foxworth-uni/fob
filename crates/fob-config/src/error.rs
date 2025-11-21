//! Error types for configuration validation and loading.

use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Error)]
pub enum ConfigError {
    // Filesystem validation errors (for CLI use)
    #[error("entry path not found: {0}")]
    EntryNotFound(PathBuf),

    #[error("plugin path not found: {0}")]
    PluginNotFound(PathBuf),

    #[error("cache directory is not writable: {0}")]
    CacheDirNotWritable(PathBuf),

    // Config parsing/loading errors
    #[error("config not found")]
    NotFound,

    #[error("unsupported configuration format: {0}")]
    UnsupportedFormat(String),

    #[error("invalid config value: {0}")]
    InvalidValue(String),

    #[error("invalid profile override: {0}")]
    InvalidProfileOverride(String),

    // Schema validation errors (no filesystem checks)
    #[error("no entries specified")]
    NoEntries,

    #[error("schema validation failed: {0}")]
    SchemaValidation(String),

    // Config evaluation errors (JS/TS execution)
    #[error("config evaluation failed: {0}")]
    EvaluationFailed(String),

    // I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

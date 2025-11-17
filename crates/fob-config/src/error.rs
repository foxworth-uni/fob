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

#[cfg(feature = "eval")]
impl From<funtime::RuntimeError> for ConfigError {
    fn from(err: funtime::RuntimeError) -> Self {
        use funtime::RuntimeError;
        match err {
            RuntimeError::ExecutionFailed(msg) => {
                ConfigError::EvaluationFailed(format!("Execution error: {}", msg))
            }
            RuntimeError::TimeoutExceeded => {
                ConfigError::EvaluationFailed("Config loading timeout (5s)".into())
            }
            RuntimeError::MemoryLimitExceeded => ConfigError::EvaluationFailed(
                "Config loading memory limit exceeded (128 MB)".into(),
            ),
            RuntimeError::PermissionDenied(msg) => {
                ConfigError::EvaluationFailed(format!("Permission denied: {}", msg))
            }
            RuntimeError::InvalidCode(msg) => {
                ConfigError::EvaluationFailed(format!("Syntax error: {}", msg))
            }
            RuntimeError::ModuleNotFound(path) => {
                ConfigError::EvaluationFailed(format!("Module not found: {}", path.display()))
            }
            RuntimeError::RuntimeUnhealthy(msg) => {
                ConfigError::EvaluationFailed(format!("Runtime unhealthy: {}", msg))
            }
            RuntimeError::Io(e) => ConfigError::Io(e),
            RuntimeError::Internal(msg) => {
                ConfigError::EvaluationFailed(format!("Internal error: {}", msg))
            }
        }
    }
}

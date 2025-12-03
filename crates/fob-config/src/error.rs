//! Error types for configuration validation and loading.

use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Error, Diagnostic)]
pub enum ConfigError {
    // Filesystem validation errors (for CLI use)
    #[error("Entry path not found: {}", path.display())]
    #[diagnostic(
        code(fob::config::entry_not_found),
        help("Check that the entry file exists at the specified path")
    )]
    EntryNotFound { path: PathBuf },

    #[error("Plugin path not found: {}", path.display())]
    #[diagnostic(
        code(fob::config::plugin_not_found),
        help("Ensure the plugin is installed and the path is correct")
    )]
    PluginNotFound { path: PathBuf },

    #[error("Cache directory is not writable: {}", path.display())]
    #[diagnostic(
        code(fob::config::cache_not_writable),
        help("Check directory permissions or specify a different cache location")
    )]
    CacheDirNotWritable { path: PathBuf },

    // Config parsing/loading errors
    #[error("Configuration file not found")]
    #[diagnostic(
        code(fob::config::not_found),
        help("Create a fob.config.json, fob.config.toml, or add a 'fob' field to package.json")
    )]
    NotFound,

    #[error("Unsupported configuration format: {format}")]
    #[diagnostic(
        code(fob::config::unsupported_format),
        help("Supported formats: .json, .toml, package.json")
    )]
    UnsupportedFormat { format: String },

    #[error("Invalid configuration value for '{field}'")]
    #[diagnostic(code(fob::config::invalid_value))]
    InvalidValue {
        field: String,
        #[help]
        hint: Option<String>,
    },

    #[error("Invalid profile override: {message}")]
    #[diagnostic(
        code(fob::config::invalid_profile),
        help("Check profile syntax: --profile.key=value")
    )]
    InvalidProfileOverride { message: String },

    // Schema validation errors (no filesystem checks)
    #[error("No entries specified in configuration")]
    #[diagnostic(
        code(fob::config::no_entries),
        help("Add at least one entry point in your config: entries: [\"./src/index.ts\"]")
    )]
    NoEntries,

    #[error("Schema validation failed: {message}")]
    #[diagnostic(code(fob::config::schema_validation))]
    SchemaValidation {
        message: String,
        #[help]
        hint: Option<String>,
    },

    // Config evaluation errors (JS/TS execution)
    #[error("Configuration evaluation failed: {message}")]
    #[diagnostic(
        code(fob::config::evaluation_failed),
        help("Check your config file for syntax errors or invalid JavaScript/TypeScript")
    )]
    EvaluationFailed { message: String },

    // I/O errors
    #[error("I/O error: {source}")]
    #[diagnostic(code(fob::config::io_error))]
    Io {
        #[source]
        #[from]
        source: std::io::Error,
    },
}

impl ConfigError {
    /// Create an InvalidValue error with a hint
    pub fn invalid_value(field: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::InvalidValue {
            field: field.into(),
            hint: Some(hint.into()),
        }
    }

    /// Create a SchemaValidation error with a hint
    pub fn schema_validation(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::SchemaValidation {
            message: message.into(),
            hint: Some(hint.into()),
        }
    }
}

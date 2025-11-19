//! Comprehensive error handling for the Joy CLI.
//!
//! This module provides a hierarchical error type system using `thiserror` for
//! structured error handling with excellent error messages. Each error variant
//! is designed to be actionable and provide context to help users resolve issues.
//!
//! # Architecture
//!
//! The error hierarchy follows these principles:
//! - **Top-level errors** (`CliError`) represent broad categories of failures
//! - **Domain-specific errors** (`ConfigError`, `BuildError`) provide detailed context
//! - **Error conversion** is automatic via `#[from]` attributes
//! - **Context helpers** allow attaching additional information to errors
//!
//! # Example
//!
//! ```rust,no_run
//! use fob_cli::error::{Result, ResultExt, CliError};
//! use std::path::Path;
//! use std::str::FromStr;
//!
//! struct Config;
//!
//! impl FromStr for Config {
//!     type Err = CliError;
//!
//!     fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
//!         Ok(Config)
//!     }
//! }
//!
//! fn load_config(path: &Path) -> Result<Config> {
//!     std::fs::read_to_string(path)
//!         .with_path(path)?
//!         .parse()
//!         .with_hint("Check JSON syntax")
//! }
//! ```

use std::path::PathBuf;
use thiserror::Error;

/// Top-level CLI error type.
///
/// This is the primary error type returned by CLI commands. It automatically
/// converts from domain-specific errors via `From` implementations.
#[derive(Debug, Error)]
pub enum CliError {
    /// Configuration-related errors (file not found, invalid syntax, etc.)
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Build process errors (missing entry points, asset failures, etc.)
    #[error("Build error: {0}")]
    Build(#[from] BuildError),

    /// Invalid command-line arguments or options
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// File or directory not found
    #[error("File not found: {}", .0.display())]
    FileNotFound(PathBuf),

    /// I/O errors from file system operations
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Development server errors
    #[error("Server error: {0}")]
    Server(String),

    /// File watching errors
    #[error("File watcher error: {0}")]
    Watch(#[from] notify::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Errors from the core bundler
    #[error("Core bundler error: {0}")]
    Core(String),

    /// Generic errors with custom messages
    #[error("{0}")]
    Custom(String),
}

/// Configuration-specific errors.
///
/// These errors occur during config file loading, parsing, and validation.
/// Each variant provides specific guidance on what went wrong.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Config file doesn't exist at the expected location
    #[error("Config file not found: {}\n\nHint: Create a fob.config.json file or specify --config <path>", .0.display())]
    NotFound(PathBuf),

    /// Config file has invalid JSON syntax
    #[error("Invalid JSON in config file: {0}\n\nHint: Use a JSON validator to check syntax")]
    InvalidJson(#[from] serde_json::Error),

    /// Config file fails JSON schema validation
    #[error("Schema validation failed:\n{errors}\n\nHint: Run 'fob config validate' to see detailed errors")]
    ValidationFailed {
        /// Formatted validation error messages
        errors: String,
    },

    /// Requested profile doesn't exist in config
    #[error("Profile '{0}' not found in config\n\nHint: Available profiles can be listed with 'fob config list-profiles'")]
    ProfileNotFound(String),

    /// Mutually exclusive options were specified
    #[error("Conflicting options: {0}\n\nHint: These options cannot be used together")]
    ConflictingOptions(String),

    /// Missing required configuration field
    #[error("Missing required field: {field}\n\nHint: {hint}")]
    MissingField {
        /// Name of the missing field
        field: String,
        /// Helpful hint for providing the field
        hint: String,
    },

    /// Invalid value for a configuration option
    #[error("Invalid value for '{field}': {value}\n\nHint: {hint}")]
    InvalidValue {
        /// Name of the field with invalid value
        field: String,
        /// The invalid value
        value: String,
        /// Helpful hint for correct values
        hint: String,
    },

    /// I/O error while reading config
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),
}

/// Build process errors.
///
/// These errors occur during the bundling process, from entry point resolution
/// to asset generation.
#[derive(Debug, Error)]
pub enum BuildError {
    /// Entry point file doesn't exist
    #[error("Entry point not found: {}\n\nHint: Check the 'entry' field in your config or --entry argument", .0.display())]
    EntryNotFound(PathBuf),

    /// Failed to write output file or asset
    #[error("Failed to write asset: {0}\n\nHint: Check output directory permissions")]
    AssetWriteFailed(String),

    /// Invalid external dependency specification
    #[error("External dependency '{0}' is invalid\n\nHint: External dependencies should be package names or URL patterns")]
    InvalidExternal(String),

    /// Module resolution failed
    #[error("Failed to resolve module: {module}\n\nImported from: {}\n\nHint: {hint}", .importer.display())]
    ResolutionFailed {
        /// The module specifier that couldn't be resolved
        module: String,
        /// The file that tried to import it
        importer: PathBuf,
        /// Helpful hint for resolution
        hint: String,
    },

    /// Circular dependency detected
    #[error("Circular dependency detected:\n{cycle}\n\nHint: Refactor to remove circular imports")]
    CircularDependency {
        /// Formatted cycle path
        cycle: String,
    },

    /// Transform/transpilation error
    #[error("Transform error in {}: {error}\n\nHint: {hint}", .file.display())]
    TransformError {
        /// File that failed to transform
        file: PathBuf,
        /// The transformation error
        error: String,
        /// Helpful hint for fixing
        hint: String,
    },

    /// Invalid source map
    #[error("Source map error: {0}\n\nHint: Disable source maps with --no-sourcemap or check input source maps")]
    SourceMapError(String),

    /// Output directory is not writable
    #[error("Output directory is not writable: {}\n\nHint: Check directory permissions or specify a different --outdir", .0.display())]
    OutputNotWritable(PathBuf),

    /// Generic build error
    #[error("{0}")]
    Custom(String),
}

/// Result type alias using `CliError` as the default error type.
///
/// This simplifies function signatures throughout the CLI.
pub type Result<T, E = CliError> = std::result::Result<T, E>;

/// Extension trait for adding context to `Result` types.
///
/// This trait provides convenient methods for enriching errors with additional
/// context like file paths or helpful hints.
pub trait ResultExt<T> {
    /// Add a file path to the error context.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use std::path::Path;
    /// # use fob_cli::error::{Result, ResultExt};
    /// # fn run() -> Result<()> {
    /// let path = Path::new("non_existent_file.txt");
    /// std::fs::read_to_string(path)
    ///     .with_path(path)?;
    /// # Ok(())
    /// # }
    /// ```
    fn with_path(self, path: impl AsRef<std::path::Path>) -> Result<T>;

    /// Add a helpful hint to the error context.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use fob_cli::error::{Result, ResultExt, CliError};
    /// # fn run() -> Result<()> {
    /// fn parse_config(content: &str) -> Result<()> {
    ///     Err(CliError::Custom("parsing failed".into()))
    /// }
    /// let content = r#"{ "key": "value" }"#;
    /// parse_config(&content)
    ///     .with_hint("Check for trailing commas in JSON")?;
    /// # Ok(())
    /// # }
    /// ```
    fn with_hint(self, hint: impl std::fmt::Display) -> Result<T>;

    /// Convert to a custom error message.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use fob_cli::error::{Result, ResultExt, CliError};
    /// # fn run() -> Result<()> {
    /// fn operation() -> Result<()> {
    ///     Err(CliError::Custom("something went wrong".into()))
    /// }
    /// operation()
    ///     .context("Failed to initialize bundler")?;
    /// # Ok(())
    /// # }
    /// ```
    fn context(self, msg: impl std::fmt::Display) -> Result<T>;
}

impl<T, E: Into<CliError>> ResultExt<T> for std::result::Result<T, E> {
    fn with_path(self, path: impl AsRef<std::path::Path>) -> Result<T> {
        self.map_err(|e| {
            let err: CliError = e.into();
            // Enhance the error with path information if it's an I/O error
            match err {
                CliError::Io(io_err) if io_err.kind() == std::io::ErrorKind::NotFound => {
                    CliError::FileNotFound(path.as_ref().to_path_buf())
                }
                other => other,
            }
        })
    }

    fn with_hint(self, hint: impl std::fmt::Display) -> Result<T> {
        self.map_err(|e| {
            let err: CliError = e.into();
            // Wrap in custom error with hint
            CliError::Custom(format!("{}\n\nHint: {}", err, hint))
        })
    }

    fn context(self, msg: impl std::fmt::Display) -> Result<T> {
        self.map_err(|e| {
            let err: CliError = e.into();
            CliError::Custom(format!("{}: {}", msg, err))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_not_found() {
        let err = ConfigError::NotFound(PathBuf::from("fob.config.json"));
        let msg = err.to_string();
        assert!(msg.contains("Config file not found"));
        assert!(msg.contains("fob.config.json"));
        assert!(msg.contains("Hint:"));
    }

    #[test]
    fn test_config_error_profile_not_found() {
        let err = ConfigError::ProfileNotFound("production".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Profile 'production' not found"));
        assert!(msg.contains("Hint:"));
    }

    #[test]
    fn test_build_error_entry_not_found() {
        let err = BuildError::EntryNotFound(PathBuf::from("src/index.ts"));
        let msg = err.to_string();
        assert!(msg.contains("Entry point not found"));
        assert!(msg.contains("src/index.ts"));
        assert!(msg.contains("Hint:"));
    }

    #[test]
    fn test_build_error_resolution_failed() {
        let err = BuildError::ResolutionFailed {
            module: "@/components/Button".to_string(),
            importer: PathBuf::from("src/App.tsx"),
            hint: "Check your path aliases in config".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Failed to resolve module"));
        assert!(msg.contains("@/components/Button"));
        assert!(msg.contains("src/App.tsx"));
        assert!(msg.contains("Hint:"));
    }

    #[test]
    fn test_cli_error_from_config_error() {
        let config_err = ConfigError::NotFound(PathBuf::from("test.json"));
        let cli_err: CliError = config_err.into();
        assert!(matches!(cli_err, CliError::Config(_)));
    }

    #[test]
    fn test_cli_error_from_build_error() {
        let build_err = BuildError::EntryNotFound(PathBuf::from("index.ts"));
        let cli_err: CliError = build_err.into();
        assert!(matches!(cli_err, CliError::Build(_)));
    }

    #[test]
    fn test_result_ext_with_path() {
        let result: std::io::Result<()> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));

        let err = result.with_path("/test/path.txt").unwrap_err();
        assert!(matches!(err, CliError::FileNotFound(_)));
    }

    #[test]
    fn test_result_ext_with_hint() {
        let result: std::result::Result<(), ConfigError> =
            Err(ConfigError::NotFound(PathBuf::from("test.json")));

        let err = result.with_hint("Try creating the file").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Hint: Try creating the file"));
    }

    #[test]
    fn test_result_ext_context() {
        let result: std::result::Result<(), ConfigError> =
            Err(ConfigError::NotFound(PathBuf::from("test.json")));

        let err = result.context("Failed to initialize").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Failed to initialize"));
    }

    #[test]
    fn test_config_error_missing_field() {
        let err = ConfigError::MissingField {
            field: "entry".to_string(),
            hint: "Add 'entry' field to your config".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Missing required field: entry"));
        assert!(msg.contains("Hint: Add 'entry' field"));
    }

    #[test]
    fn test_config_error_invalid_value() {
        let err = ConfigError::InvalidValue {
            field: "format".to_string(),
            value: "invalid".to_string(),
            hint: "Must be 'esm' or 'cjs'".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Invalid value for 'format'"));
        assert!(msg.contains("invalid"));
        assert!(msg.contains("Must be 'esm' or 'cjs'"));
    }

    #[test]
    fn test_build_error_circular_dependency() {
        let err = BuildError::CircularDependency {
            cycle: "A -> B -> C -> A".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Circular dependency"));
        assert!(msg.contains("A -> B -> C -> A"));
    }
}

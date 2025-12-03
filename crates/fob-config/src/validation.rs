//! Pluggable config validation strategies
//!
//! Separates filesystem validation (for CLI use) from schema validation (for library use).

use std::path::Path;

use crate::bundle::BundleOptions;
use crate::error::{ConfigError, Result};

/// Trait for pluggable config validation strategies
pub trait ConfigValidator {
    /// Validate bundle options
    fn validate(&self, config: &BundleOptions) -> Result<()>;
}

/// Schema-only validation (no filesystem checks)
///
/// Use this for library/SaaS use cases where files are in-memory or virtual.
///
/// # Example
///
/// ```
/// use fob_config::{BundleOptions, SchemaValidator, ConfigValidator};
///
/// let mut config = BundleOptions::default();
/// config.entries = vec!["index.mdx".into()];
///
/// let validator = SchemaValidator;
/// validator.validate(&config).unwrap();
/// ```
pub struct SchemaValidator;

impl ConfigValidator for SchemaValidator {
    fn validate(&self, config: &BundleOptions) -> Result<()> {
        // Entry validation
        if config.entries.is_empty() {
            return Err(ConfigError::NoEntries);
        }

        // Validate external packages (must be non-empty strings)
        for external in &config.external {
            if external.trim().is_empty() {
                return Err(ConfigError::SchemaValidation {
                    message: "external package names cannot be empty".to_string(),
                    hint: Some("Remove empty strings from the 'external' array".to_string()),
                });
            }
        }

        // Validate plugin configurations
        for plugin in &config.plugins {
            if plugin.path.as_os_str().is_empty() {
                return Err(ConfigError::SchemaValidation {
                    message: "plugin path cannot be empty".to_string(),
                    hint: Some("Specify a valid path for each plugin".to_string()),
                });
            }

            // Validate order is reasonable
            if plugin.order < -1000 || plugin.order > 1000 {
                return Err(ConfigError::SchemaValidation {
                    message: format!(
                        "plugin order {} is out of reasonable range (-1000 to 1000)",
                        plugin.order
                    ),
                    hint: Some("Use an order value between -1000 and 1000".to_string()),
                });
            }
        }

        // Validate cache config
        if config.cache_config.max_size > 0 && config.cache_config.max_size < 1024 * 1024 {
            return Err(ConfigError::SchemaValidation {
                message: "cache max_size must be at least 1MB (1048576 bytes) or 0 for unlimited"
                    .to_string(),
                hint: Some("Set max_size to 0 for unlimited or at least 1048576".to_string()),
            });
        }

        Ok(())
    }
}

/// Filesystem validator (for CLI use)
///
/// Validates that entry points, plugin paths, and cache directories exist on disk.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use fob_config::{BundleOptions, FsValidator, ConfigValidator};
///
/// let mut config = BundleOptions::default();
/// config.entries = vec!["src/index.ts".into()];
///
/// let validator = FsValidator::new(".");
/// validator.validate(&config).unwrap();
/// ```
pub struct FsValidator {
    root: std::path::PathBuf,
}

impl FsValidator {
    /// Create a new filesystem validator with a root directory
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }
}

impl ConfigValidator for FsValidator {
    fn validate(&self, config: &BundleOptions) -> Result<()> {
        // First run schema validation
        SchemaValidator.validate(config)?;

        // Then validate filesystem references
        for entry in &config.entries {
            let path = self.root.join(entry);
            if !path.exists() {
                return Err(ConfigError::EntryNotFound { path });
            }
        }

        if let Some(dir) = &config.cache_config.cache_dir {
            let path = self.root.join(dir);
            if !path.exists() {
                return Err(ConfigError::CacheDirNotWritable { path });
            }
        }

        for plugin in &config.plugins {
            let path = self.root.join(&plugin.path);
            if !path.exists() {
                return Err(ConfigError::PluginNotFound { path });
            }
        }

        Ok(())
    }
}

/// Convenience function for schema-only validation
///
/// # Example
///
/// ```
/// use fob_config::{BundleOptions, validate_schema};
///
/// let mut config = BundleOptions::default();
/// config.entries = vec!["index.mdx".into()];
///
/// validate_schema(&config).unwrap();
/// ```
pub fn validate_schema(config: &BundleOptions) -> Result<()> {
    SchemaValidator.validate(config)
}

/// Convenience function for filesystem validation
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use fob_config::{BundleOptions, validate_fs};
///
/// let mut config = BundleOptions::default();
/// config.entries = vec!["src/index.ts".into()];
///
/// validate_fs(&config, ".").unwrap();
/// ```
pub fn validate_fs(config: &BundleOptions, root: impl AsRef<Path>) -> Result<()> {
    FsValidator::new(root).validate(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn schema_validator_rejects_empty_entries() {
        let config = BundleOptions::default(); // No entries
        let result = SchemaValidator.validate(&config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::NoEntries));
    }

    #[test]
    fn schema_validator_accepts_valid_config() {
        let mut config = BundleOptions::default();
        config.entries = vec![PathBuf::from("index.ts")];
        let result = SchemaValidator.validate(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn schema_validator_rejects_empty_external() {
        let mut config = BundleOptions::default();
        config.entries = vec![PathBuf::from("index.ts")];
        config.external = vec!["react".to_string(), "   ".to_string()]; // Empty after trim
        let result = SchemaValidator.validate(&config);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::SchemaValidation { .. }
        ));
    }

    #[test]
    fn schema_validator_rejects_invalid_plugin_order() {
        let mut config = BundleOptions::default();
        config.entries = vec![PathBuf::from("index.ts")];
        config.plugins = vec![crate::bundle::PluginOptions {
            name: Some("test".to_string()),
            backend: None,
            path: PathBuf::from("plugin.wasm"),
            config: serde_json::Value::Null,
            order: 9999, // Out of range
            enabled: true,
            pool_size: None,
            max_memory_bytes: None,
            timeout_ms: None,
            profiles: std::collections::HashMap::new(),
        }];
        let result = SchemaValidator.validate(&config);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::SchemaValidation { .. }
        ));
    }

    #[test]
    fn schema_validator_rejects_tiny_cache_size() {
        let mut config = BundleOptions::default();
        config.entries = vec![PathBuf::from("index.ts")];
        config.cache_config.max_size = 1024; // Less than 1MB
        let result = SchemaValidator.validate(&config);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::SchemaValidation { .. }
        ));
    }

    #[test]
    fn validate_schema_helper_works() {
        let mut config = BundleOptions::default();
        config.entries = vec![PathBuf::from("index.ts")];
        assert!(validate_schema(&config).is_ok());
    }
}

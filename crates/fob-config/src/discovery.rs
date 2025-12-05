//! File-based config discovery for CLI use
//!
//! Handles finding and loading Fob configuration files from the filesystem.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::config::JoyConfig;
use crate::error::{ConfigError, Result};

/// File-based configuration discovery
///
/// Searches for Fob configuration files in conventional locations and loads them.
/// This is primarily for CLI use - library users should use `JoyConfig::from_value()` directly.
///
/// # Example
///
/// ```no_run
/// use fob_config::ConfigDiscovery;
///
/// let discovery = ConfigDiscovery::new(".");
/// let config = discovery.load().unwrap();
/// ```
pub struct ConfigDiscovery {
    root: PathBuf,
}

impl ConfigDiscovery {
    /// Create a new config discovery with a root directory
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    /// Find a config file in the root directory
    ///
    /// Searches in this order:
    /// 1. TOML config: fob.toml
    /// 2. package.json (fob field)
    pub fn find(&self) -> Option<PathBuf> {
        let toml_path = self.root.join("fob.toml");
        if toml_path.exists() {
            return Some(toml_path);
        }

        // package.json with fob field
        let pkg_path = self.root.join("package.json");
        if pkg_path.exists() {
            if let Ok(content) = fs::read_to_string(&pkg_path) {
                if let Ok(parsed) = serde_json::from_str::<Value>(&content) {
                    if parsed.get("fob").is_some() && !parsed["fob"].is_null() {
                        return Some(pkg_path);
                    }
                }
            }
        }

        None
    }

    /// Load config from discovered file
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::NotFound` if no config file is found.
    pub fn load(&self) -> Result<JoyConfig> {
        let path = self.find().ok_or(ConfigError::NotFound)?;
        self.load_from(&path)
    }

    /// Load config with profile merging
    pub fn load_with_profile(&self, profile: &str) -> Result<JoyConfig> {
        let mut config = self.load()?;
        config = config.materialize_profile(Some(profile))?;
        Ok(config)
    }

    /// Load config from a specific file path
    fn load_from(&self, path: &Path) -> Result<JoyConfig> {
        // Handle package.json specially
        if path.file_name() == Some(std::ffi::OsStr::new("package.json")) {
            return self.load_from_package_json(path);
        }

        let content = fs::read_to_string(path)?;

        let toml_val: toml::Value =
            toml::from_str(&content).map_err(|e| ConfigError::InvalidValue {
                field: "toml".to_string(),
                hint: Some(format!("Invalid TOML syntax: {}", e)),
            })?;

        let value = serde_json::to_value(toml_val).map_err(|e| ConfigError::InvalidValue {
            field: "toml".to_string(),
            hint: Some(format!("TOML to JSON conversion failed: {}", e)),
        })?;

        JoyConfig::from_value(value)
    }

    fn load_from_package_json(&self, path: &Path) -> Result<JoyConfig> {
        let content = fs::read_to_string(path)?;

        let parsed: Value =
            serde_json::from_str(&content).map_err(|e| ConfigError::InvalidValue {
                field: "package.json".to_string(),
                hint: Some(format!("Invalid JSON: {}", e)),
            })?;

        let fob_value = parsed.get("fob").ok_or_else(|| ConfigError::InvalidValue {
            field: "fob".to_string(),
            hint: Some("Add a 'fob' field to your package.json".to_string()),
        })?;

        if fob_value.is_null() {
            return Err(ConfigError::InvalidValue {
                field: "fob".to_string(),
                hint: Some("The 'fob' field cannot be null".to_string()),
            });
        }

        JoyConfig::from_value(fob_value.clone())
    }
}

/// Discover and load config from current directory (convenience function)
///
/// # Example
///
/// ```no_run
/// use fob_config::discover;
///
/// let config = discover().unwrap();
/// ```
pub fn discover() -> Result<JoyConfig> {
    #[cfg(not(target_arch = "wasm32"))]
    let root = std::env::current_dir()?;

    #[cfg(target_arch = "wasm32")]
    let root = PathBuf::from("/");

    ConfigDiscovery::new(&root).load()
}

/// Discover and load config with profile (convenience function)
///
/// # Example
///
/// ```no_run
/// use fob_config::discover_with_profile;
///
/// let config = discover_with_profile("production").unwrap();
/// ```
pub fn discover_with_profile(profile: &str) -> Result<JoyConfig> {
    #[cfg(not(target_arch = "wasm32"))]
    let root = std::env::current_dir()?;

    #[cfg(target_arch = "wasm32")]
    let root = PathBuf::from("/");

    ConfigDiscovery::new(&root).load_with_profile(profile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn find_returns_none_when_no_config() {
        let dir = TempDir::new().unwrap();
        let discovery = ConfigDiscovery::new(dir.path());
        assert!(discovery.find().is_none());
    }

    #[test]
    fn find_discovers_toml_config() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("fob.toml");
        fs::write(
            &config_path,
            r#"
[bundle]
entries = ["index.ts"]
"#,
        )
        .unwrap();

        let discovery = ConfigDiscovery::new(dir.path());
        assert_eq!(discovery.find().unwrap(), config_path);
    }

    #[test]
    fn load_returns_not_found_when_no_config() {
        let dir = TempDir::new().unwrap();
        let discovery = ConfigDiscovery::new(dir.path());
        let result = discovery.load();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::NotFound));
    }

    #[test]
    fn load_parses_toml_config() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("fob.toml"),
            r#"
[bundle]
entries = ["index.ts"]
minify = true
"#,
        )
        .unwrap();

        let discovery = ConfigDiscovery::new(dir.path());
        let config = discovery.load().unwrap();
        assert_eq!(config.bundle.entries, vec![PathBuf::from("index.ts")]);
        assert!(config.bundle.minify);
    }

    #[test]
    fn load_from_package_json() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{
                "name": "test",
                "fob": {
                    "bundle": {
                        "entries": ["index.ts"]
                    }
                }
            }"#,
        )
        .unwrap();

        let discovery = ConfigDiscovery::new(dir.path());
        let config = discovery.load().unwrap();
        assert_eq!(config.bundle.entries, vec![PathBuf::from("index.ts")]);
    }
}

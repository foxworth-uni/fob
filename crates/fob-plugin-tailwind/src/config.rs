//! Tailwind CSS configuration types
//!
//! This module provides configuration options for the Tailwind CSS plugin.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for Tailwind CSS processing
///
/// This matches the structure of `tailwind.config.js` but in Rust types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TailwindConfig {
    /// Content paths to scan for class names
    ///
    /// Glob patterns for files to scan (e.g., "./src/**/*.{js,jsx,ts,tsx}")
    pub content: Vec<String>,

    /// Theme customization
    pub theme: ThemeConfig,

    /// Plugins to load
    pub plugins: Vec<String>,

    /// Dark mode configuration
    #[serde(rename = "darkMode")]
    pub dark_mode: DarkModeConfig,

    /// Package manager to use for running Tailwind CLI
    ///
    /// Supported values: "pnpm", "npm", "bun", "deno"
    /// If not specified, auto-detects from lockfiles (pnpm-lock.yaml, package-lock.json, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_manager: Option<String>,

    /// Optional path to Tailwind config file
    ///
    /// If specified, will be passed to the CLI via `--config` flag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_file: Option<PathBuf>,

    /// Enable CSS minification
    ///
    /// When true, passes `--minify` to the Tailwind CLI
    #[serde(default)]
    pub minify: bool,

    /// Additional CLI arguments to pass to Tailwind
    ///
    /// These will be appended to the generated command
    #[serde(default)]
    pub cli_args: Vec<String>,
}

impl Default for TailwindConfig {
    fn default() -> Self {
        Self {
            content: vec![
                "./src/**/*.{js,jsx,ts,tsx}".to_string(),
                "./index.html".to_string(),
            ],
            theme: ThemeConfig::default(),
            plugins: Vec::new(),
            dark_mode: DarkModeConfig::Class,
            package_manager: None,
            config_file: None,
            minify: false,
            cli_args: Vec::new(),
        }
    }
}

impl TailwindConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the package manager to use
    ///
    /// # Arguments
    ///
    /// * `pm` - Package manager name: "pnpm", "npm", "bun", or "deno"
    pub fn with_package_manager(mut self, pm: impl Into<String>) -> Self {
        self.package_manager = Some(pm.into());
        self
    }

    /// Set the config file path
    pub fn with_config_file(mut self, path: PathBuf) -> Self {
        self.config_file = Some(path);
        self
    }

    /// Enable minification
    pub fn with_minify(mut self, enabled: bool) -> Self {
        self.minify = enabled;
        self
    }

    /// Add a CLI argument
    pub fn with_cli_arg(mut self, arg: impl Into<String>) -> Self {
        self.cli_args.push(arg.into());
        self
    }
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Extend or override default theme
    pub extend: serde_json::Value,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            extend: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

/// Dark mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DarkModeConfig {
    /// Use media query
    Media,
    /// Use class-based dark mode
    Class,
    /// Disabled
    False,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TailwindConfig::default();
        assert!(!config.content.is_empty());
        assert_eq!(config.plugins.len(), 0);
    }

    #[test]
    fn test_serialization() {
        let config = TailwindConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("content"));
    }
}

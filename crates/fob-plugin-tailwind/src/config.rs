//! Tailwind CSS configuration types

use std::path::PathBuf;

/// Configuration for Tailwind CSS plugin
#[derive(Debug, Clone, Default)]
pub struct TailwindConfig {
    /// Package manager to use: "pnpm", "npm", "bun", "deno"
    /// If None, auto-detects from lockfiles
    pub package_manager: Option<String>,

    /// Path to tailwind.config.js (optional, CLI auto-detects)
    pub config_file: Option<PathBuf>,

    /// Enable CSS minification
    pub minify: bool,
}

impl TailwindConfig {
    /// Set the package manager
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
}

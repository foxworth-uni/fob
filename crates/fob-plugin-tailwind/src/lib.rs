//! Rolldown plugin implementation for Tailwind CSS.
//!
//! This module provides a Rolldown plugin that integrates Tailwind CSS processing
//! into the Rolldown bundler pipeline. It uses the `load` hook to intercept
//! `.css` files with `@tailwind` directives and process them using the Tailwind CLI.
//!
//! ## Architecture
//!
//! ```text
//! CSS file → load() → contains @tailwind? → YES → process with CLI (file path) → CSS
//!                                   ↓
//!                                   NO → skip (let other plugins handle)
//! ```
//!
//! ## Plugin Order
//!
//! This plugin should be registered BEFORE the CSS plugin so it can claim
//! `@tailwind` files. Files without `@tailwind` directives fall through to
//! the CSS plugin for processing with lightningcss.
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use fob_plugin_tailwind::FobTailwindPlugin;
//! use std::sync::Arc;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Use with your Rolldown bundler configuration
//! let plugin = Arc::new(FobTailwindPlugin::new(PathBuf::from(".")));
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use fob_bundler::{HookLoadArgs, HookLoadOutput, HookLoadReturn, ModuleType, Plugin, PluginContext};
use std::borrow::Cow;
use std::path::Path;
use std::sync::Arc;
use tokio::runtime::Handle;
use tracing::debug;

mod config;
mod error;
mod generator;

pub use config::TailwindConfig;
pub use error::GeneratorError;
pub use generator::{PackageManager, TailwindGenerator};

/// Rolldown plugin that processes Tailwind CSS
///
/// This plugin:
/// 1. Scans for CSS files with `@tailwind` directives.
/// 2. Invokes the Tailwind CSS CLI to process them, which handles content
///    scanning and CSS generation based on the project's `tailwind.config.js`.
#[derive(Clone, Debug)]
pub struct FobTailwindPlugin {
    /// Configuration options for Tailwind CSS
    config: TailwindConfig,

    /// Lazily initialized Tailwind CSS generator
    /// Uses tokio::sync::OnceCell for async-aware lazy initialization
    generator: Arc<tokio::sync::OnceCell<TailwindGenerator>>,

    /// Project root directory for resolving paths
    project_root: std::path::PathBuf,
}

impl FobTailwindPlugin {
    /// Create a new FobTailwindPlugin with default configuration
    ///
    /// # Example
    ///
    /// ```rust
    /// use fob_plugin_tailwind::FobTailwindPlugin;
    /// use std::path::PathBuf;
    ///
    /// let plugin = FobTailwindPlugin::new(PathBuf::from("."));
    /// ```
    pub fn new(project_root: std::path::PathBuf) -> Self {
        Self {
            config: TailwindConfig::default(),
            generator: Arc::new(tokio::sync::OnceCell::new()),
            project_root,
        }
    }

    /// Create a new FobTailwindPlugin with custom configuration
    ///
    /// # Example
    ///
    /// ```rust
    /// use fob_plugin_tailwind::{FobTailwindPlugin, TailwindConfig};
    /// use std::path::PathBuf;
    ///
    /// let config = TailwindConfig::default();
    /// let plugin = FobTailwindPlugin::with_config(config, PathBuf::from("."));
    /// ```
    pub fn with_config(config: TailwindConfig, project_root: std::path::PathBuf) -> Self {
        Self {
            config,
            generator: Arc::new(tokio::sync::OnceCell::new()),
            project_root,
        }
    }

    /// Get or initialize the Tailwind generator
    ///
    /// This method lazily creates the generator on first call and reuses it
    /// for subsequent calls.
    async fn get_generator(&self) -> Result<&TailwindGenerator, GeneratorError> {
        self.generator
            .get_or_try_init(|| async {
                // Create generator based on config
                let generator = if let Some(pm_str) = &self.config.package_manager {
                    // Parse package manager string
                    let pm = match pm_str.to_lowercase().as_str() {
                        "pnpm" => PackageManager::Pnpm,
                        "npm" => PackageManager::Npm,
                        "bun" => PackageManager::Bun,
                        "deno" => PackageManager::Deno,
                        _ => {
                            return Err(GeneratorError::cli_not_found(vec![self
                                .project_root
                                .join("package.json")]));
                        }
                    };
                    TailwindGenerator::with_package_manager(pm, self.project_root.clone()).await?
                } else {
                    // Auto-detect package manager
                    TailwindGenerator::new(self.project_root.clone()).await?
                };

                // Apply config options
                let mut generator = generator;
                if let Some(config_file) = &self.config.config_file {
                    generator = generator.with_config(config_file.clone());
                }
                if self.config.minify {
                    generator = generator.with_minify(true);
                }

                Ok(generator)
            })
            .await
    }

    /// Process CSS file with @tailwind directives using file path
    ///
    /// This method passes the file path directly to the Tailwind CLI instead
    /// of using stdin. This is more reliable as some CLI versions don't
    /// properly handle stdin input.
    async fn process_css_file(&self, path: &Path) -> Result<String> {
        // Get or initialize the generator
        let generator = self
            .get_generator()
            .await
            .context("Failed to initialize Tailwind generator")?;

        // Generate CSS from the file path using the CLI
        let generated_css = generator
            .generate_from_file(path)
            .await
            .context("Failed to generate CSS from Tailwind CLI")?;

        Ok(generated_css)
    }
}

impl Default for FobTailwindPlugin {
    fn default() -> Self {
        Self::new(std::path::PathBuf::from("."))
    }
}

impl Plugin for FobTailwindPlugin {
    /// Returns the plugin name for debugging and logging
    fn name(&self) -> Cow<'static, str> {
        "fob-tailwind".into()
    }

    /// Declare which hooks this plugin uses
    fn register_hook_usage(&self) -> fob_bundler::HookUsage {
        fob_bundler::HookUsage::Load
    }

    /// Load hook - processes CSS files with `@tailwind` directives.
    ///
    /// This hook runs BEFORE the CSS plugin's load hook (if registered first).
    /// It peeks at CSS files and claims those with `@tailwind` directives,
    /// processing them directly with the Tailwind CLI using the file path.
    ///
    /// Files without `@tailwind` directives are skipped (return None),
    /// allowing them to fall through to the CSS plugin.
    fn load(
        &self,
        _ctx: &PluginContext,
        args: &HookLoadArgs<'_>,
    ) -> impl std::future::Future<Output = HookLoadReturn> + Send {
        let id = args.id.to_string();
        let plugin = self.clone();

        async move {
            // Only handle .css files
            if !id.ends_with(".css") {
                return Ok(None);
            }

            // Peek at file to check for @tailwind directives
            let content = match std::fs::read_to_string(&id) {
                Ok(c) => c,
                Err(_) => return Ok(None), // Let other plugins handle if we can't read
            };

            if !content.contains("@tailwind") {
                // No @tailwind directives - let CSS plugin handle it
                return Ok(None);
            }

            debug!("[fob-tailwind] Loading @tailwind CSS file: {}", id);

            // Process with Tailwind CLI using FILE PATH (not stdin)
            let path = std::path::PathBuf::from(&id);
            let handle = Handle::try_current();

            let processed_result = match handle {
                Ok(h) => {
                    h.spawn(async move { plugin.process_css_file(&path).await })
                        .await?
                }
                Err(_) => {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()?;
                    rt.block_on(async move { plugin.process_css_file(&path).await })
                }
            };

            let processed = processed_result
                .with_context(|| format!("Failed to process Tailwind CSS: {}", id))?;

            debug!(
                "[fob-tailwind] Loaded {} ({} bytes output)",
                id,
                processed.len()
            );

            Ok(Some(HookLoadOutput {
                code: processed.into(),
                module_type: Some(ModuleType::Css),
                ..Default::default()
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_plugin_creation() {
        let plugin = FobTailwindPlugin::new(PathBuf::from("."));
        assert_eq!(plugin.name(), "fob-tailwind");
    }

    #[test]
    fn test_plugin_with_custom_config() {
        let config = TailwindConfig::default();
        let plugin = FobTailwindPlugin::with_config(config, PathBuf::from("."));
        assert_eq!(plugin.name(), "fob-tailwind");
    }

    #[test]
    fn test_config_builder() {
        let config = TailwindConfig::default()
            .with_package_manager("pnpm")
            .with_minify(true);

        assert_eq!(config.package_manager, Some("pnpm".to_string()));
        assert!(config.minify);
    }
}

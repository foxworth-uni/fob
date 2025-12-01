//! Rolldown plugin implementation for Tailwind CSS.
//!
//! This module provides a Rolldown plugin that integrates Tailwind CSS processing
//! into the Rolldown bundler pipeline. It uses the `transform` hook to intercept
//! `.css` files and process `@tailwind` directives using the Tailwind CLI.
//!
//! ## Architecture
//!
//! ```text
//! CSS file → transform() → contains @tailwind? → YES → process with CLI → transformed CSS
//!                                         ↓
//!                                         NO → skip
//! ```
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
use fob_bundler::{
    HookTransformArgs, HookTransformOutput, HookTransformReturn, Plugin,
    SharedTransformPluginContext,
};
use std::borrow::Cow;
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

    /// Process CSS file with @tailwind directives
    ///
    /// Replaces @tailwind directives with actual Tailwind CSS utilities
    /// by invoking the Tailwind CLI.
    async fn process_css(&self, content: &str) -> Result<String> {
        // Get or initialize the generator
        let generator = self
            .get_generator()
            .await
            .context("Failed to initialize Tailwind generator")?;

        // Generate CSS from the input CSS content using the CLI
        let generated_css = generator
            .generate_from_input(content)
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
        use fob_bundler::HookUsage;
        HookUsage::Transform
    }

    /// Transform hook - processes CSS files with `@tailwind` directives.
    ///
    /// It no longer scans source files for classes, delegating that responsibility
    /// entirely to the Tailwind CLI, which reads the `content` configuration
    /// from `tailwind.config.js`.
    fn transform(
        &self,
        _ctx: SharedTransformPluginContext,
        args: &HookTransformArgs<'_>,
    ) -> impl std::future::Future<Output = HookTransformReturn> + Send {
        let id = args.id.to_string();
        let code = args.code.to_string();

        // Clone fields needed for the async block
        let plugin = self.clone();

        async move {
            // Only handle CSS files with @tailwind directives
            if !id.ends_with(".css") || !code.contains("@tailwind") {
                return Ok(None);
            }

            debug!("[fob-tailwind] Processing @tailwind directives in: {}", id);

            // In order to not block the main tokio runtime, we need to spawn this
            // on a blocking thread if we are in a sync context.
            let handle = Handle::try_current();

            let code_len = code.len();
            let processed_css_result = match handle {
                Ok(h) => {
                    // We are in an async context, just await it.
                    h.spawn(async move { plugin.process_css(&code).await })
                        .await?
                }
                Err(_) => {
                    // We are in a sync context, spawn a new runtime to handle this.
                    // This is a fallback and might be less efficient.
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()?;
                    rt.block_on(async move { plugin.process_css(&code).await })
                }
            };

            let processed = processed_css_result
                .with_context(|| format!("Failed to process Tailwind CSS in: {}", id))?;

            debug!(
                "[fob-tailwind] Processed {} ({} -> {} bytes)",
                id,
                code_len,
                processed.len()
            );

            // Return processed CSS
            Ok(Some(HookTransformOutput {
                code: Some(processed),
                map: None,
                side_effects: None,
                module_type: None,
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

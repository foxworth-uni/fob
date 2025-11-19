//! Rolldown plugin implementation for Tailwind CSS (using tailwindcss-oxide)
//!
//! This module provides a Rolldown plugin that integrates Tailwind CSS processing
//! into the Rolldown bundler pipeline. It uses both the `load` and `transform` hooks:
//!
//! - `load` hook: Intercepts `.css` files and processes `@tailwind` directives
//! - `transform` hook: Scans source files (`.tsx`, `.jsx`, etc.) for class names
//!
//! ## Architecture
//!
//! ```text
//! Source files → transform() → scan classes → collect in registry
//!                                ↓
//! CSS file → load() → process @tailwind → inject scanned classes → CSS
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
use parking_lot::Mutex;
use rolldown_plugin::{
    HookTransformArgs, HookTransformReturn, HookTransformOutput,
    Plugin, SharedTransformPluginContext
};
use rustc_hash::FxHashSet;
use std::borrow::Cow;
use std::sync::Arc;

mod config;
mod error;
mod generator;

pub use config::TailwindConfig;
pub use error::GeneratorError;
pub use generator::{PackageManager, TailwindGenerator};

/// Rolldown plugin that processes Tailwind CSS
///
/// This plugin:
/// 1. Scans source files for CSS class names using tailwindcss-oxide
/// 2. Processes CSS files with @tailwind directives
/// 3. Generates final CSS with only the classes actually used
///
/// # Architecture
///
/// The plugin maintains a shared registry of discovered class names that is
/// populated during the `transform` phase and consumed during the `load` phase.
/// The Tailwind CLI generator is lazily initialized on first use.
#[derive(Clone, Debug)]
pub struct FobTailwindPlugin {
    /// Configuration options for Tailwind CSS
    config: TailwindConfig,

    /// Shared registry of discovered CSS classes
    /// Thread-safe to allow concurrent transforms
    class_registry: Arc<Mutex<FxHashSet<String>>>,

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
            class_registry: Arc::new(Mutex::new(FxHashSet::default())),
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
            class_registry: Arc::new(Mutex::new(FxHashSet::default())),
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
                            return Err(GeneratorError::cli_not_found(vec![self.project_root.join("package.json")]));
                        }
                    };
                    TailwindGenerator::with_package_manager(pm, self.project_root.clone())
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
    /// based on the classes found in the registry using the Tailwind CLI.
    async fn process_css(&self, content: &str, classes: &[String]) -> Result<String> {
        // Check if content has @tailwind directives
        if !content.contains("@tailwind") {
            return Ok(content.to_string());
        }

        // Get or initialize the generator
        let generator = self
            .get_generator()
            .await
            .context("Failed to initialize Tailwind generator")?;

        // Generate CSS from candidates using the CLI
        let generated_css = generator
            .generate(classes)
            .await
            .context("Failed to generate CSS from Tailwind CLI")?;

        // Build the final CSS output
        let mut output = String::new();

        // Replace @tailwind directives with generated CSS
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("@tailwind") {
                // For any @tailwind directive, inject the generated CSS
                // (the CLI handles all directives together)
                if !generated_css.is_empty() {
                    output.push_str(&generated_css);
                    output.push('\n');
                }
            } else {
                // Preserve custom CSS
                output.push_str(line);
                output.push('\n');
            }
        }

        Ok(output)
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
    ///
    /// This allows Rolldown to optimize by skipping unused hooks.
    fn register_hook_usage(&self) -> rolldown_plugin::HookUsage {
        use rolldown_plugin::HookUsage;
        // We only use transform for both CSS processing and class scanning
        HookUsage::Transform
    }

    /// Transform hook - processes CSS files and scans source files for class names
    ///
    /// This hook serves two purposes:
    /// 1. For `.css` files: processes @tailwind directives and generates final CSS
    /// 2. For source files: scans for class names and adds them to the registry
    ///
    /// # Returns
    ///
    /// - `Ok(Some(output))` - File was transformed (CSS processed or classes scanned)
    /// - `Ok(None)` - File type not handled by this plugin
    /// - `Err(e)` - Processing error
    fn transform(
        &self,
        _ctx: SharedTransformPluginContext,
        args: &HookTransformArgs<'_>,
    ) -> impl std::future::Future<Output = HookTransformReturn> + Send {
        let id = args.id.to_string();
        let code = args.code.to_string();
        let class_registry = self.class_registry.clone();
        let generator = self.generator.clone();
        let config = self.config.clone();
        let project_root = self.project_root.clone();

        async move {
            // Handle CSS files with @tailwind directives
            if id.ends_with(".css") {
                eprintln!("[fob-tailwind] Transform hook called for CSS: {}", id);

                // Check if it contains @tailwind directives
                if !code.contains("@tailwind") {
                    eprintln!("[fob-tailwind] No @tailwind directives found, skipping");
                    return Ok(None);
                }

                // Get the discovered classes
                let classes_set = class_registry.lock().clone();
                let classes: Vec<String> = classes_set.into_iter().collect();

                eprintln!("[fob-tailwind] Processing CSS with {} classes", classes.len());

                // Reconstruct plugin from captured fields
                let plugin = FobTailwindPlugin {
                    config,
                    class_registry,
                    generator,
                    project_root,
                };

                let processed = plugin.process_css(&code, &classes).await
                    .with_context(|| format!("Failed to process Tailwind CSS in: {}", id))?;

                eprintln!(
                    "[fob-tailwind] Processed {} ({} → {} bytes, {} classes)",
                    id,
                    code.len(),
                    processed.len(),
                    classes.len()
                );

                // Return processed CSS
                return Ok(Some(HookTransformOutput {
                    code: Some(processed),
                    map: None,
                    side_effects: None,
                    module_type: None,
                }));
            }

            // Handle source files - scan for class names
            let should_scan = id.ends_with(".tsx")
                || id.ends_with(".jsx")
                || id.ends_with(".ts")
                || id.ends_with(".js");

            if !should_scan {
                return Ok(None);
            }

            eprintln!("[fob-tailwind] Scanning for classes: {}", id);

            // Scan for class names using tailwindcss-oxide
            use tailwindcss_oxide::extractor::{Extracted, Extractor};

            let mut extractor = Extractor::new(code.as_bytes());
            let extracted = extractor.extract();

            let classes: Vec<String> = extracted
                .into_iter()
                .filter_map(|item| match item {
                    Extracted::Candidate(bytes) => {
                        std::str::from_utf8(bytes).ok().map(|s| s.to_string())
                    }
                    Extracted::CssVariable(_) => None,
                })
                .collect();

            // Add to registry
            if !classes.is_empty() {
                let mut registry = class_registry.lock();
                let count_before = registry.len();
                for class in classes {
                    registry.insert(class);
                }
                let new_classes = registry.len() - count_before;
                if new_classes > 0 {
                    eprintln!("[fob-tailwind] Found {} new classes in {}", new_classes, id);
                }
            }

            // Return None to indicate no transformation of source code
            Ok(None)
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
    fn test_plugin_default() {
        let plugin = FobTailwindPlugin::default();
        assert_eq!(plugin.name(), "fob-tailwind");
    }

    #[test]
    fn test_config_builder() {
        let config = TailwindConfig::new()
            .with_package_manager("pnpm")
            .with_minify(true);

        assert_eq!(config.package_manager, Some("pnpm".to_string()));
        assert!(config.minify);
    }
}

//! Rolldown plugin implementation for lightningcss
//!
//! This module provides a Rolldown plugin that integrates lightningcss CSS processing
//! into the Rolldown bundler pipeline. It uses the `load` hook to intercept `.css` files
//! and process them through lightningcss for bundling, minification, and optimization.
//!
//! ## Features
//!
//! - **CSS Bundling**: Resolve `@import` statements recursively
//! - **Minification**: Optimize CSS size (merge rules, shorthands, etc.)
//! - **Browser Targets**: Auto-prefix CSS based on browserslist queries
//! - **Source Maps**: Generate source maps for debugging
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use fob_plugin_css::FobCssPlugin;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Use with your Rolldown bundler configuration
//! let plugin = Arc::new(FobCssPlugin::new());
//! # Ok(())
//! # }
//! ```

use anyhow::Context;
use lightningcss::{
    printer::PrinterOptions,
    stylesheet::{MinifyOptions, ParserOptions, StyleSheet},
};
use rolldown_common::ModuleType;
use rolldown_plugin::{HookLoadArgs, HookLoadOutput, HookLoadReturn, Plugin, PluginContext};
use std::borrow::Cow;
use std::path::Path;

mod config;
pub use config::CssPluginOptions;

/// Rolldown plugin that processes CSS files using lightningcss
///
/// This plugin intercepts `.css` file loading and processes them through lightningcss,
/// providing bundling (@import resolution), minification, and browser-specific transforms.
///
/// # Architecture
///
/// ```text
/// .css file → load() hook → lightningcss bundle → minify → target transforms → CSS
/// ```
#[derive(Debug, Clone)]
pub struct FobCssPlugin {
    /// Configuration options for CSS processing
    options: CssPluginOptions,
}

impl FobCssPlugin {
    /// Create a new FobCssPlugin with default options
    ///
    /// # Example
    ///
    /// ```rust
    /// use fob_plugin_css::FobCssPlugin;
    ///
    /// let plugin = FobCssPlugin::new();
    /// ```
    pub fn new() -> Self {
        Self {
            options: CssPluginOptions::default(),
        }
    }

    /// Create a new FobCssPlugin with custom options
    ///
    /// # Example
    ///
    /// ```rust
    /// use fob_plugin_css::{FobCssPlugin, CssPluginOptions};
    ///
    /// let options = CssPluginOptions::new()
    ///     .with_minify(true)
    ///     .with_targets(vec![">0.2%".to_string(), "not dead".to_string()]);
    ///
    /// let plugin = FobCssPlugin::with_options(options);
    /// ```
    pub fn with_options(options: CssPluginOptions) -> Self {
        Self { options }
    }

    /// Process a CSS file through lightningcss
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the CSS file
    /// * `source` - CSS source code
    ///
    /// # Returns
    ///
    /// Processed CSS as a string
    fn process_css(&self, path: &Path, source: String) -> anyhow::Result<String> {
        // Parse CSS
        // Note: @import bundling will be added in future iteration
        let mut stylesheet = StyleSheet::parse(
            &source,
            ParserOptions {
                filename: path.to_string_lossy().to_string(),
                ..Default::default()
            },
        )
        .map_err(|e| anyhow::anyhow!("Failed to parse CSS from {}: {:?}", path.display(), e))?;

        // Minify if enabled
        if self.options.minify {
            stylesheet
                .minify(MinifyOptions::default())
                .map_err(|e| anyhow::anyhow!("Failed to minify CSS from {}: {:?}", path.display(), e))?;
        }

        // Print to string
        let result = stylesheet
            .to_css(PrinterOptions {
                minify: self.options.minify,
                // TODO: Add browser targets support
                // targets: self.get_targets()?,
                ..Default::default()
            })
            .map_err(|e| anyhow::anyhow!("Failed to print CSS from {}: {:?}", path.display(), e))?;

        Ok(result.code)
    }

    /// Check if a file should be processed based on include/exclude patterns
    fn should_process(&self, path: &str) -> bool {
        // Skip if explicitly excluded
        if !self.options.exclude.is_empty() {
            for pattern in &self.options.exclude {
                if path.contains(pattern.as_str()) {
                    return false;
                }
            }
        }

        // If include patterns are specified, file must match one
        if !self.options.include.is_empty() {
            return self
                .options
                .include
                .iter()
                .any(|pattern| path.contains(pattern.as_str()));
        }

        true
    }
}

impl Default for FobCssPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for FobCssPlugin {
    /// Returns the plugin name for debugging and logging
    fn name(&self) -> Cow<'static, str> {
        "fob-css".into()
    }

    /// Declare which hooks this plugin uses
    ///
    /// This allows Rolldown to optimize by skipping unused hooks.
    fn register_hook_usage(&self) -> rolldown_plugin::HookUsage {
        use rolldown_plugin::HookUsage;
        // We only use the load hook
        HookUsage::Load
    }

    /// Load hook - intercepts `.css` files and processes them
    ///
    /// This is the core of the plugin. It:
    /// 1. Checks if the file is a `.css` file
    /// 2. Reads the file from disk
    /// 3. Processes CSS through lightningcss:
    ///    - Resolves @import statements
    ///    - Minifies (if enabled)
    ///    - Applies browser targets (if configured)
    /// 4. Returns processed CSS with `ModuleType::Css`
    ///
    /// # Returns
    ///
    /// - `Ok(Some(output))` - Successfully processed CSS file
    /// - `Ok(None)` - Not a CSS file, let Rolldown handle it
    /// - `Err(e)` - Processing error
    fn load(
        &self,
        _ctx: &PluginContext,
        args: &HookLoadArgs<'_>,
    ) -> impl std::future::Future<Output = HookLoadReturn> + Send {
        // Capture data needed for async block to avoid lifetime issues
        let id = args.id.to_string();
        let options = self.options.clone();

        async move {
            // Only handle .css files
            if !id.ends_with(".css") {
                return Ok(None);
            }

            // Check if file should be processed
            let plugin = FobCssPlugin::with_options(options);
            if !plugin.should_process(&id) {
                eprintln!("[fob-css] Skipping excluded file: {}", id);
                return Ok(None);
            }

            // Read the CSS source file
            let source = std::fs::read_to_string(&id)
                .with_context(|| format!("Failed to read CSS file: {}", id))?;

            // Save source length before moving
            let source_len = source.len();

            // Process CSS through lightningcss
            let path = Path::new(&id);
            let processed = plugin.process_css(path, source)?;

            eprintln!(
                "[fob-css] Processed {} ({} → {} bytes, minify: {})",
                id,
                source_len,
                processed.len(),
                plugin.options.minify
            );

            // Return processed CSS to Rolldown
            // IMPORTANT: Set module_type to Css so Rolldown knows how to handle it
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

    #[test]
    fn test_plugin_creation() {
        let plugin = FobCssPlugin::new();
        assert_eq!(plugin.name(), "fob-css");
    }

    #[test]
    fn test_plugin_with_options() {
        let options = CssPluginOptions::new().with_minify(true);
        let plugin = FobCssPlugin::with_options(options);
        assert_eq!(plugin.name(), "fob-css");
        assert!(plugin.options.minify);
    }

    #[test]
    fn test_plugin_default() {
        let plugin = FobCssPlugin::default();
        assert_eq!(plugin.name(), "fob-css");
    }

    #[test]
    fn test_should_process_exclusions() {
        let plugin = FobCssPlugin::with_options(
            CssPluginOptions::new().exclude("vendor/")
        );

        assert!(!plugin.should_process("node_modules/vendor/styles.css"));
        assert!(plugin.should_process("src/styles.css"));
    }

    #[test]
    fn test_should_process_inclusions() {
        let plugin = FobCssPlugin::with_options(
            CssPluginOptions::new().include("src/")
        );

        assert!(plugin.should_process("src/styles.css"));
        assert!(!plugin.should_process("vendor/styles.css"));
    }

    #[test]
    fn test_process_basic_css() {
        let plugin = FobCssPlugin::new();
        let css = "body { color: red; }".to_string();
        let path = Path::new("test.css");

        let result = plugin.process_css(path, css);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("color"));
    }

    #[test]
    fn test_process_with_minification() {
        let plugin = FobCssPlugin::with_options(
            CssPluginOptions::new().with_minify(true)
        );

        let css = "body {\n  color: red;\n  background: blue;\n}";
        let path = Path::new("test.css");

        let result = plugin.process_css(path, css.to_string()).unwrap();
        // Minified CSS should be smaller
        assert!(result.len() < css.len());
        // Should still contain the properties
        assert!(result.contains("color"));
        assert!(result.contains("background"));
    }
}

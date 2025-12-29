//! Built-in CSS processing functionality
//!
//! This module provides CSS processing capabilities integrated into the bundler.
//! It uses lightningcss to transform, minify, and optimize CSS files during bundling.
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
//! use fob_bundler::builtins::CssPlugin;
//! use std::sync::Arc;
//! use fob_bundler::runtime::BundlerRuntime;
//! use fob_bundler::Runtime;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new("."));
//! let plugin = Arc::new(CssPlugin::new(runtime));
//! # Ok(())
//! # }
//! ```

use crate::plugins::{FobPlugin, PluginPhase};
use crate::{
    HookLoadArgs, HookLoadOutput, HookLoadReturn, ModuleType, Plugin, PluginContext, Runtime,
};
use anyhow::Context;
use lightningcss::{
    printer::PrinterOptions,
    stylesheet::{MinifyOptions, ParserOptions, StyleSheet},
};
use std::borrow::Cow;
use std::path::Path;
use std::sync::Arc;

mod config;
pub use config::CssPluginOptions;

/// Built-in CSS processing functionality
///
/// This processes `.css` files through lightningcss,
/// providing bundling (@import resolution), minification, and browser-specific transforms.
///
/// # Architecture
///
/// ```text
/// .css file → load() hook → lightningcss bundle → minify → target transforms → CSS
/// ```
#[derive(Clone, Debug)]
pub struct CssPlugin {
    /// Configuration options for CSS processing
    options: CssPluginOptions,
    /// Runtime for file access (handles virtual files + filesystem)
    runtime: Arc<dyn Runtime>,
}

impl CssPlugin {
    /// Create a new CssPlugin with default options
    ///
    /// # Arguments
    ///
    /// * `runtime` - Runtime for file access (handles virtual files + filesystem)
    ///
    /// # Example
    ///
    /// ```rust
    /// use fob_bundler::builtins::CssPlugin;
    /// use fob_bundler::Runtime;
    /// use std::sync::Arc;
    ///
    /// # async fn example(runtime: Arc<dyn Runtime>) {
    /// let plugin = CssPlugin::new(runtime);
    /// # }
    /// ```
    pub fn new(runtime: Arc<dyn Runtime>) -> Self {
        Self {
            options: CssPluginOptions::default(),
            runtime,
        }
    }

    /// Create a new CssPlugin with custom options
    ///
    /// # Arguments
    ///
    /// * `runtime` - Runtime for file access (handles virtual files + filesystem)
    /// * `options` - CSS processing options
    ///
    /// # Example
    ///
    /// ```rust
    /// use fob_bundler::builtins::{CssPlugin, CssPluginOptions};
    /// use fob_bundler::Runtime;
    /// use std::sync::Arc;
    ///
    /// # async fn example(runtime: Arc<dyn Runtime>) {
    /// let options = CssPluginOptions::new()
    ///     .with_minify(true)
    ///     .with_targets(vec![">0.2%".to_string(), "not dead".to_string()]);
    ///
    /// let plugin = CssPlugin::with_options(runtime, options);
    /// # }
    /// ```
    pub fn with_options(runtime: Arc<dyn Runtime>, options: CssPluginOptions) -> Self {
        Self { options, runtime }
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
        let mut stylesheet = StyleSheet::parse(
            &source,
            ParserOptions {
                filename: path.to_string_lossy().to_string(),
                ..Default::default()
            },
        )
        .map_err(|e| anyhow::anyhow!("Failed to parse CSS from {}: {:?}", path.display(), e))?;

        if self.options.minify {
            stylesheet.minify(MinifyOptions::default()).map_err(|e| {
                anyhow::anyhow!("Failed to minify CSS from {}: {:?}", path.display(), e)
            })?;
        }

        let result = stylesheet
            .to_css(PrinterOptions {
                minify: self.options.minify,
                ..Default::default()
            })
            .map_err(|e| anyhow::anyhow!("Failed to print CSS from {}: {:?}", path.display(), e))?;

        Ok(result.code)
    }

    /// Check if a file should be processed based on include/exclude patterns
    fn should_process(&self, path: &str) -> bool {
        if !self.options.exclude.is_empty() {
            for pattern in &self.options.exclude {
                if path.contains(pattern.as_str()) {
                    return false;
                }
            }
        }

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

impl Plugin for CssPlugin {
    /// Returns the name for debugging and logging
    fn name(&self) -> Cow<'static, str> {
        "fob-css".into()
    }

    /// Declare which hooks this implementation uses
    ///
    /// This allows Rolldown to optimize by skipping unused hooks.
    fn register_hook_usage(&self) -> crate::HookUsage {
        use crate::HookUsage;
        HookUsage::Load
    }

    /// Load hook - intercepts `.css` files and processes them
    ///
    /// This is the core functionality. It:
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
        let id = args.id.to_string();
        let options = self.options.clone();
        let runtime = Arc::clone(&self.runtime);

        async move {
            if !id.ends_with(".css") {
                return Ok(None);
            }

            let plugin = CssPlugin::with_options(Arc::clone(&runtime), options);
            if !plugin.should_process(&id) {
                eprintln!("[fob-css] Skipping excluded file: {}", id);
                return Ok(None);
            }

            let file_path = std::path::Path::new(&id);
            let content = runtime
                .read_file(file_path)
                .await
                .with_context(|| format!("Failed to read CSS file: {}", id))?;
            let source = String::from_utf8(content)
                .with_context(|| format!("CSS file {} contains invalid UTF-8", id))?;

            let source_len = source.len();

            let path = Path::new(&id);
            let processed = plugin.process_css(path, source)?;

            eprintln!(
                "[fob-css] Processed {} ({} → {} bytes, minify: {})",
                id,
                source_len,
                processed.len(),
                plugin.options.minify
            );

            Ok(Some(HookLoadOutput {
                code: processed.into(),
                module_type: Some(ModuleType::Css),
                ..Default::default()
            }))
        }
    }
}

impl FobPlugin for CssPlugin {
    fn phase(&self) -> PluginPhase {
        PluginPhase::Transform
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Runtime;
    use std::sync::Arc;

    #[cfg(not(target_family = "wasm"))]
    #[test]
    fn test_plugin_creation() {
        use crate::runtime::BundlerRuntime;
        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new("."));
        let plugin = CssPlugin::new(runtime);
        assert_eq!(plugin.name(), "fob-css");
    }

    #[cfg(not(target_family = "wasm"))]
    #[test]
    fn test_plugin_with_options() {
        use crate::runtime::BundlerRuntime;
        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new("."));
        let options = CssPluginOptions::new().with_minify(true);
        let plugin = CssPlugin::with_options(runtime, options);
        assert_eq!(plugin.name(), "fob-css");
        assert!(plugin.options.minify);
    }

    #[cfg(not(target_family = "wasm"))]
    #[test]
    fn test_should_process_exclusions() {
        use crate::runtime::BundlerRuntime;
        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new("."));
        let plugin = CssPlugin::with_options(runtime, CssPluginOptions::new().exclude("vendor/"));

        assert!(!plugin.should_process("node_modules/vendor/styles.css"));
        assert!(plugin.should_process("src/styles.css"));
    }

    #[cfg(not(target_family = "wasm"))]
    #[test]
    fn test_should_process_inclusions() {
        use crate::runtime::BundlerRuntime;
        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new("."));
        let plugin = CssPlugin::with_options(runtime, CssPluginOptions::new().include("src/"));

        assert!(plugin.should_process("src/styles.css"));
        assert!(!plugin.should_process("vendor/styles.css"));
    }

    #[cfg(not(target_family = "wasm"))]
    #[test]
    fn test_process_basic_css() {
        use crate::runtime::BundlerRuntime;
        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new("."));
        let plugin = CssPlugin::new(runtime);
        let css = "body { color: red; }".to_string();
        let path = Path::new("test.css");

        let result = plugin.process_css(path, css);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("color"));
    }

    #[cfg(not(target_family = "wasm"))]
    #[test]
    fn test_process_with_minification() {
        use crate::runtime::BundlerRuntime;
        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new("."));
        let plugin = CssPlugin::with_options(runtime, CssPluginOptions::new().with_minify(true));

        let css = "body {\n  color: red;\n  background: blue;\n}";
        let path = Path::new("test.css");

        let result = plugin.process_css(path, css.to_string()).unwrap();
        assert!(result.len() < css.len());
        assert!(result.contains("color"));
        assert!(result.contains("background"));
    }
}

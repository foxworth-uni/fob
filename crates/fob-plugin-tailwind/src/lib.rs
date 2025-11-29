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
use fob_bundler::{
    HookTransformArgs, HookTransformOutput, HookTransformReturn, Plugin,
    SharedTransformPluginContext,
};
use parking_lot::Mutex;
use rustc_hash::FxHashSet;
use std::borrow::Cow;
use std::sync::Arc;
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

/// Check if a file path should be scanned based on exclusion patterns
fn should_scan_file_path(id: &str) -> bool {
    // Skip node_modules and vendor directories
    if id.contains("/node_modules/")
        || id.contains("/.pnpm/")
        || id.contains("/dist/")
        || id.contains("/vendor/")
        || id.contains("/.cache/")
        || id.contains("\\node_modules\\")
        || id.contains("\\.pnpm\\")
        || id.contains("\\dist\\")
        || id.contains("\\vendor\\")
        || id.contains("\\.cache\\")
    {
        return false;
    }
    true
}

/// Check if a file matches content patterns from config
fn matches_content_patterns(id: &str, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        // If no patterns specified, allow all matching extensions
        return true;
    }

    // Simple glob pattern matching for common patterns
    // This handles basic patterns like "./src/**/*.{js,jsx,ts,tsx}"
    for pattern in patterns {
        // Normalize pattern: remove leading "./" and handle **
        let normalized_pattern = pattern.trim_start_matches("./");

        // Simple matching: check if the file path contains the base directory
        // For patterns like "src/**/*.{js,jsx}", check if path starts with "src/"
        if normalized_pattern.contains("**") {
            let base = normalized_pattern.split("**").next().unwrap_or("");
            if !base.is_empty() && id.contains(base) {
                return true;
            }
        } else if id.contains(normalized_pattern.trim_end_matches("*")) {
            return true;
        }
    }

    false
}

/// Validate that a class name is a valid CSS class (not a JS keyword/identifier)
fn is_valid_css_class(class: &str) -> bool {
    // Reject empty strings
    if class.is_empty() {
        return false;
    }

    // Reject JavaScript keywords and common identifiers
    let js_keywords = [
        "class",
        "function",
        "const",
        "let",
        "var",
        "if",
        "else",
        "for",
        "while",
        "return",
        "import",
        "export",
        "default",
        "async",
        "await",
        "try",
        "catch",
        "throw",
        "new",
        "this",
        "super",
        "extends",
        "implements",
        "interface",
        "type",
        "enum",
        "namespace",
        "module",
        "declare",
        "abstract",
        "static",
        "public",
        "private",
        "protected",
        "readonly",
        "get",
        "set",
        "of",
        "in",
        "instanceof",
        "typeof",
        "void",
        "null",
        "undefined",
        "true",
        "false",
        "break",
        "continue",
        "switch",
        "case",
        "do",
        "with",
        "debugger",
        "yield",
    ];

    // Check if it's a JS keyword (case-insensitive)
    if js_keywords.iter().any(|&kw| class.eq_ignore_ascii_case(kw)) {
        return false;
    }

    // Reject identifiers that look like JS (camelCase function names, PascalCase classes)
    // But allow valid Tailwind classes that might be camelCase
    // A simple heuristic: reject if it's purely alphanumeric and starts with uppercase
    // (likely a class name like "Component" or "React")
    if class
        .chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
        && class.chars().all(|c| c.is_alphanumeric() || c == '_')
        && class.len() > 1
        && !class.contains('-')
        && !class.contains(':')
        && !class.contains('.')
        && !class.contains('/')
        && !class.contains('[')
        && !class.contains(']')
    {
        // Likely a JS identifier, but allow if it contains Tailwind-specific characters
        return false;
    }

    // Allow classes that contain Tailwind-specific characters (:, /, [, ], etc.)
    // or contain hyphens (common in CSS)
    if class.contains('-') || class.contains(':') || class.contains('/') || class.contains('[') {
        return true;
    }

    // For simple identifiers, be more permissive but reject obvious JS patterns
    // Allow if it's a valid CSS identifier pattern
    true
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
                            return Err(GeneratorError::cli_not_found(vec![self
                                .project_root
                                .join("package.json")]));
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
    fn register_hook_usage(&self) -> fob_bundler::HookUsage {
        use fob_bundler::HookUsage;
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
                debug!("[fob-tailwind] Transform hook called for CSS: {}", id);

                // Check if it contains @tailwind directives
                if !code.contains("@tailwind") {
                    debug!("[fob-tailwind] No @tailwind directives found, skipping");
                    return Ok(None);
                }

                // Get the discovered classes
                let classes_set = class_registry.lock().clone();
                let classes: Vec<String> = classes_set.into_iter().collect();

                debug!(
                    "[fob-tailwind] Processing CSS with {} classes",
                    classes.len()
                );

                // Reconstruct plugin from captured fields
                let plugin = FobTailwindPlugin {
                    config,
                    class_registry,
                    generator,
                    project_root,
                };

                let processed = plugin
                    .process_css(&code, &classes)
                    .await
                    .with_context(|| format!("Failed to process Tailwind CSS in: {}", id))?;

                debug!(
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
            // First check file extension
            let has_valid_extension = id.ends_with(".tsx")
                || id.ends_with(".jsx")
                || id.ends_with(".ts")
                || id.ends_with(".js")
                || id.ends_with(".html")
                || id.ends_with(".vue")
                || id.ends_with(".svelte")
                || id.ends_with(".astro")
                || id.ends_with(".mdx");

            if !has_valid_extension {
                return Ok(None);
            }

            // Check path exclusions (node_modules, etc.)
            if !should_scan_file_path(&id) {
                debug!("[fob-tailwind] Skipping excluded path: {}", id);
                return Ok(None);
            }

            // Check content patterns from config
            if !matches_content_patterns(&id, &config.content) {
                debug!(
                    "[fob-tailwind] File does not match content patterns: {}",
                    id
                );
                return Ok(None);
            }

            debug!("[fob-tailwind] Scanning for classes: {}", id);

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
                .filter(|class| is_valid_css_class(class))
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
                    debug!("[fob-tailwind] Found {} new classes in {}", new_classes, id);
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

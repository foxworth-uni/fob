//! Rolldown plugin implementation for Tailwind CSS v4.
//!
//! This module provides a Rolldown plugin that integrates Tailwind CSS v4 processing
//! into the Rolldown bundler pipeline. It uses the `load` hook to intercept
//! `.css` files with Tailwind v4 `@import "tailwindcss"` syntax and process them
//! using the Tailwind CLI.
//!
//! ## Architecture
//!
//! ```text
//! CSS file → load() → detect syntax → v3? → ERROR (not supported)
//!                                   ↓
//!                                  v4? → YES → process with CLI (file path) → CSS
//!                                   ↓
//!                                  NO → skip (let other plugins handle)
//! ```
//!
//! ## Plugin Order
//!
//! This plugin should be registered BEFORE the CSS plugin so it can claim
//! Tailwind v4 files. Files without Tailwind syntax fall through to
//! the CSS plugin for processing with lightningcss.
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use fob_plugin_tailwind::FobTailwindPlugin;
//! use fob_bundler::{Runtime, runtime::BundlerRuntime};
//! use std::sync::Arc;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Use with your Rolldown bundler configuration
//! let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new("."));
//! let plugin = Arc::new(FobTailwindPlugin::new(runtime, PathBuf::from(".")));
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use fob_bundler::{
    FobPlugin, HookLoadArgs, HookLoadOutput, HookLoadReturn, ModuleType, Plugin, PluginContext,
    PluginPhase, Runtime,
};
use regex::Regex;
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

/// Result of detecting Tailwind syntax in CSS content
#[derive(Debug, PartialEq)]
enum TailwindDetection {
    /// No Tailwind syntax detected
    None,
    /// Tailwind CSS v3 syntax detected (not supported)
    V3(Vec<String>),
    /// Tailwind CSS v4 syntax detected (supported)
    V4,
}

/// Detect Tailwind CSS syntax in CSS content
///
/// This function checks for both v3 and v4 syntax patterns:
/// - v3: `@tailwind base|components|utilities|variants`
/// - v4: `@import "tailwindcss"` or `@import "tailwindcss/utilities"`
///
/// Detection order is important: we check for v3 first to provide
/// helpful migration guidance if unsupported syntax is found.
fn detect_tailwind_syntax(content: &str) -> TailwindDetection {
    // Lazy static patterns using regex
    // We use lazy initialization to compile regex patterns once
    use std::sync::OnceLock;

    static V3_PATTERN: OnceLock<Regex> = OnceLock::new();
    static V4_PATTERN: OnceLock<Regex> = OnceLock::new();

    // v3 pattern: @tailwind followed by whitespace and one of the layer names
    let v3_re = V3_PATTERN
        .get_or_init(|| Regex::new(r"@tailwind\s+(base|components|utilities|variants)").unwrap());

    // v4 pattern: @import "tailwindcss" or @import "tailwindcss/utilities"
    let v4_re = V4_PATTERN
        .get_or_init(|| Regex::new(r#"@import\s+["']tailwindcss(?:/[a-z]+)?["']"#).unwrap());

    // Check v3 FIRST - we want to reject unsupported syntax early
    let v3_matches: Vec<String> = v3_re
        .captures_iter(content)
        .map(|cap| format!("@tailwind {}", &cap[1]))
        .collect();

    if !v3_matches.is_empty() {
        return TailwindDetection::V3(v3_matches);
    }

    // Check v4 syntax
    if v4_re.is_match(content) {
        return TailwindDetection::V4;
    }

    // No Tailwind syntax found
    TailwindDetection::None
}

/// Rolldown plugin that processes Tailwind CSS v4
///
/// This plugin:
/// 1. Scans for CSS files with Tailwind v4 `@import "tailwindcss"` syntax.
/// 2. Rejects Tailwind v3 `@tailwind` directives with a helpful migration message.
/// 3. Invokes the Tailwind CSS CLI to process v4 files, which handles content
///    scanning and CSS generation based on the project's configuration.
#[derive(Clone, Debug)]
pub struct FobTailwindPlugin {
    /// Configuration options for Tailwind CSS
    config: TailwindConfig,

    /// Lazily initialized Tailwind CSS generator
    /// Uses tokio::sync::OnceCell for async-aware lazy initialization
    generator: Arc<tokio::sync::OnceCell<TailwindGenerator>>,

    /// Project root directory for resolving paths
    project_root: std::path::PathBuf,

    /// Runtime for file access (handles virtual files + filesystem)
    runtime: Arc<dyn Runtime>,
}

impl FobTailwindPlugin {
    /// Create a new FobTailwindPlugin with default configuration
    ///
    /// # Arguments
    ///
    /// * `runtime` - Runtime for file access (handles virtual files + filesystem)
    /// * `project_root` - Project root directory for resolving paths
    ///
    /// # Example
    ///
    /// ```rust
    /// use fob_plugin_tailwind::FobTailwindPlugin;
    /// use fob_bundler::{Runtime, runtime::BundlerRuntime};
    /// use std::path::PathBuf;
    /// use std::sync::Arc;
    ///
    /// # async fn example() {
    /// let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new("."));
    /// let plugin = FobTailwindPlugin::new(runtime, PathBuf::from("."));
    /// # }
    /// ```
    pub fn new(runtime: Arc<dyn Runtime>, project_root: std::path::PathBuf) -> Self {
        Self {
            config: TailwindConfig::default(),
            generator: Arc::new(tokio::sync::OnceCell::new()),
            project_root,
            runtime,
        }
    }

    /// Create a new FobTailwindPlugin with custom configuration
    ///
    /// # Arguments
    ///
    /// * `runtime` - Runtime for file access (handles virtual files + filesystem)
    /// * `config` - Tailwind configuration options
    /// * `project_root` - Project root directory for resolving paths
    ///
    /// # Example
    ///
    /// ```rust
    /// use fob_plugin_tailwind::{FobTailwindPlugin, TailwindConfig};
    /// use fob_bundler::{Runtime, runtime::BundlerRuntime};
    /// use std::path::PathBuf;
    /// use std::sync::Arc;
    ///
    /// # async fn example() {
    /// let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new("."));
    /// let config = TailwindConfig::default();
    /// let plugin = FobTailwindPlugin::with_config(runtime, config, PathBuf::from("."));
    /// # }
    /// ```
    pub fn with_config(
        runtime: Arc<dyn Runtime>,
        config: TailwindConfig,
        project_root: std::path::PathBuf,
    ) -> Self {
        Self {
            config,
            generator: Arc::new(tokio::sync::OnceCell::new()),
            project_root,
            runtime,
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

// Note: Default is removed since Runtime is required

impl Plugin for FobTailwindPlugin {
    /// Returns the plugin name for debugging and logging
    fn name(&self) -> Cow<'static, str> {
        "fob-tailwind".into()
    }

    /// Declare which hooks this plugin uses
    fn register_hook_usage(&self) -> fob_bundler::HookUsage {
        fob_bundler::HookUsage::Load
    }

    /// Load hook - processes CSS files with Tailwind v4 syntax.
    ///
    /// This hook runs BEFORE the CSS plugin's load hook (if registered first).
    /// It peeks at CSS files and:
    /// - Claims those with v4 `@import "tailwindcss"` syntax for processing
    /// - Rejects those with v3 `@tailwind` directives (returns error)
    /// - Skips files without Tailwind syntax (allows fallthrough to CSS plugin)
    ///
    /// Files with v4 syntax are processed directly with the Tailwind CLI
    /// using the file path.
    fn load(
        &self,
        _ctx: &PluginContext,
        args: &HookLoadArgs<'_>,
    ) -> impl std::future::Future<Output = HookLoadReturn> + Send {
        let id = args.id.to_string();
        let plugin = self.clone();
        let runtime = Arc::clone(&self.runtime);

        async move {
            // Only handle .css files
            if !id.ends_with(".css") {
                return Ok(None);
            }

            // Peek at file to detect Tailwind syntax using Runtime
            let file_path = std::path::Path::new(&id);
            let content = match runtime.read_file(file_path).await {
                Ok(bytes) => match String::from_utf8(bytes) {
                    Ok(s) => s,
                    Err(_) => return Ok(None), // Invalid UTF-8, let other plugins handle
                },
                Err(_) => return Ok(None), // Let other plugins handle if we can't read
            };

            // Detect Tailwind syntax version
            match detect_tailwind_syntax(&content) {
                TailwindDetection::None => {
                    // No Tailwind syntax - let CSS plugin handle it
                    return Ok(None);
                }
                TailwindDetection::V3(directives) => {
                    // v3 syntax not supported - return helpful error
                    let error = GeneratorError::v3_not_supported(id.clone(), directives);
                    return Err(error.into());
                }
                TailwindDetection::V4 => {
                    // v4 syntax detected - proceed with processing
                    debug!("[fob-tailwind] Loading Tailwind v4 CSS file: {}", id);
                }
            }

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

impl FobPlugin for FobTailwindPlugin {
    fn phase(&self) -> PluginPhase {
        // Tailwind should run before CSS plugin (both Transform phase, but Tailwind has lower priority)
        // Actually, since phases are the same, order matters - Tailwind should be added first
        PluginPhase::Transform
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fob_bundler::Runtime;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[cfg(not(target_family = "wasm"))]
    #[test]
    fn test_plugin_creation() {
        use fob_bundler::runtime::BundlerRuntime;
        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new("."));
        let plugin = FobTailwindPlugin::new(runtime, PathBuf::from("."));
        assert_eq!(plugin.name(), "fob-tailwind");
    }

    #[cfg(not(target_family = "wasm"))]
    #[test]
    fn test_plugin_with_custom_config() {
        use fob_bundler::runtime::BundlerRuntime;
        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new("."));
        let config = TailwindConfig::default();
        let plugin = FobTailwindPlugin::with_config(runtime, config, PathBuf::from("."));
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

    // Tailwind syntax detection tests

    #[test]
    fn test_detects_v4_import_syntax() {
        let css = r#"
            @import "tailwindcss";

            .my-class {
                color: red;
            }
        "#;

        assert_eq!(detect_tailwind_syntax(css), TailwindDetection::V4);
    }

    #[test]
    fn test_detects_v4_granular_imports() {
        let test_cases = vec![
            r#"@import "tailwindcss/utilities";"#,
            r#"@import "tailwindcss/base";"#,
            r#"@import "tailwindcss/components";"#,
            r#"@import 'tailwindcss/utilities';"#, // single quotes
            r#"
                @import "tailwindcss/base";
                @import "tailwindcss/utilities";
            "#,
        ];

        for css in test_cases {
            assert_eq!(
                detect_tailwind_syntax(css),
                TailwindDetection::V4,
                "Failed to detect v4 syntax in: {}",
                css
            );
        }
    }

    #[test]
    fn test_rejects_v3_directives() {
        let css = r#"
            @tailwind base;
            @tailwind components;
            @tailwind utilities;

            .my-class {
                color: red;
            }
        "#;

        match detect_tailwind_syntax(css) {
            TailwindDetection::V3(directives) => {
                assert_eq!(directives.len(), 3);
                assert!(directives.contains(&"@tailwind base".to_string()));
                assert!(directives.contains(&"@tailwind components".to_string()));
                assert!(directives.contains(&"@tailwind utilities".to_string()));
            }
            other => panic!("Expected V3 detection, got: {:?}", other),
        }
    }

    #[test]
    fn test_rejects_v3_single_directive() {
        let css = r#"
            @tailwind utilities;
        "#;

        match detect_tailwind_syntax(css) {
            TailwindDetection::V3(directives) => {
                assert_eq!(directives.len(), 1);
                assert_eq!(directives[0], "@tailwind utilities");
            }
            other => panic!("Expected V3 detection, got: {:?}", other),
        }
    }

    #[test]
    fn test_rejects_v3_variants() {
        let css = r#"
            @tailwind base;
            @tailwind variants;
        "#;

        match detect_tailwind_syntax(css) {
            TailwindDetection::V3(directives) => {
                assert_eq!(directives.len(), 2);
                assert!(directives.contains(&"@tailwind base".to_string()));
                assert!(directives.contains(&"@tailwind variants".to_string()));
            }
            other => panic!("Expected V3 detection, got: {:?}", other),
        }
    }

    #[test]
    fn test_no_tailwind_returns_none() {
        let css = r#"
            .my-class {
                color: red;
                font-size: 16px;
            }

            @media (min-width: 768px) {
                .my-class {
                    font-size: 20px;
                }
            }
        "#;

        assert_eq!(detect_tailwind_syntax(css), TailwindDetection::None);
    }

    #[test]
    fn test_no_tailwind_with_other_imports() {
        let css = r#"
            @import "normalize.css";
            @import url("https://fonts.googleapis.com/css2?family=Inter");

            .my-class {
                color: red;
            }
        "#;

        assert_eq!(detect_tailwind_syntax(css), TailwindDetection::None);
    }

    #[test]
    fn test_v3_takes_precedence_over_v4() {
        // Edge case: if someone has both v3 and v4 syntax (migration in progress),
        // we should detect v3 first and error
        let css = r#"
            @tailwind base;
            @import "tailwindcss";
        "#;

        match detect_tailwind_syntax(css) {
            TailwindDetection::V3(directives) => {
                assert_eq!(directives.len(), 1);
                assert_eq!(directives[0], "@tailwind base");
            }
            other => panic!("Expected V3 detection to take precedence, got: {:?}", other),
        }
    }

    #[test]
    fn test_whitespace_handling() {
        let test_cases = vec![
            "@tailwind   base;",
            "@tailwind\tcomponents;",
            "@tailwind\nutilities;",
            "@import  \"tailwindcss\";",
            "@import\t\"tailwindcss/utilities\";",
        ];

        for css in &test_cases[..3] {
            // v3 cases
            assert!(
                matches!(detect_tailwind_syntax(css), TailwindDetection::V3(_)),
                "Failed to detect v3 with whitespace: {}",
                css
            );
        }

        for css in &test_cases[3..] {
            // v4 cases
            assert_eq!(
                detect_tailwind_syntax(css),
                TailwindDetection::V4,
                "Failed to detect v4 with whitespace: {}",
                css
            );
        }
    }
}

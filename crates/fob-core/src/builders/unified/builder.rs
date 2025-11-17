//! Builder API wrapper around BuildOptions.
//!
//! Provides an ergonomic builder pattern for configuring builds with
//! preset methods for common use cases.

use std::path::PathBuf;
use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::runtime::Runtime;
use crate::analysis::analyzer::Analyzer;
use crate::Result;

use super::entry::EntryPoints;
use super::options::BuildOptions;
use super::output::BuildResult;
use crate::builders::build_executor::execute_build;

/// Builder API for configuring and executing builds.
///
/// Provides a fluent interface for building with preset methods for
/// common use cases (library, app, components).
///
/// # Example
///
/// ```rust,no_run
/// use fob_core::builders::unified::builder::Builder;
///
/// # async fn example() -> fob_core::Result<()> {
/// // Library build
/// let result = Builder::library("src/index.ts")
///     .external(vec!["react"])
///     // .emit_dts(true) // If dts-generation feature is enabled
///     .build()
///     .await?;
///
/// result.write_to("dist").await?;
///
/// // App build with code splitting
/// let result = Builder::app(vec!["src/main.ts", "src/admin.ts"])
///     .splitting(true)
///     .minify(true)
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct Builder {
    options: BuildOptions,
}

impl Builder {
    /// Create a new builder with default options.
    pub fn new() -> Self {
        Self {
            options: BuildOptions::default(),
        }
    }

    /// Create a builder preset for a library build.
    ///
    /// Libraries typically:
    /// - Don't bundle dependencies (externalize them)
    /// - Generate TypeScript declarations
    /// - Output in multiple formats (ESM, CJS)
    pub fn library(entry: impl Into<PathBuf>) -> Self {
        Self::new()
            .entry(entry)
            .bundle(false)
    }

    /// Create a builder preset for an app build.
    ///
    /// Apps typically:
    /// - Bundle all dependencies
    /// - Support code splitting
    /// - Minify for production
    pub fn app(entries: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        let entries: Vec<PathBuf> = entries.into_iter().map(|e| e.into()).collect();
        Self::new()
            .entries(entries)
            .bundle(true)
    }

    /// Create a builder preset for component builds.
    ///
    /// Components are similar to apps but typically:
    /// - Multiple entry points (one per component)
    /// - No code splitting (each component is independent)
    pub fn components(entries: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        let entries: Vec<PathBuf> = entries.into_iter().map(|e| e.into()).collect();
        Self::new()
            .entries(entries)
            .bundle(true)
            .splitting(false)
    }

    /// Set a single entry point.
    pub fn entry(mut self, entry: impl Into<PathBuf>) -> Self {
        self.options.entry = EntryPoints::Single(entry.into().to_string_lossy().to_string());
        self
    }

    /// Set multiple entry points.
    pub fn entries(mut self, entries: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        let entries: Vec<String> = entries
            .into_iter()
            .map(|e| e.into().to_string_lossy().to_string())
            .collect();
        self.options.entry = EntryPoints::Multiple(entries);
        self
    }

    /// Set named entry points.
    pub fn named_entries(mut self, entries: FxHashMap<String, String>) -> Self {
        self.options.entry = EntryPoints::Named(entries);
        self
    }

    /// Whether to bundle dependencies (default: true).
    pub fn bundle(mut self, bundle: bool) -> Self {
        self.options.bundle = bundle;
        self
    }

    /// Enable code splitting (default: false).
    pub fn splitting(mut self, splitting: bool) -> Self {
        self.options.splitting = splitting;
        self
    }

    /// Mark packages as external (not bundled).
    pub fn external(mut self, packages: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.options.external.extend(packages.into_iter().map(|p| p.into()));
        self
    }

    /// Set output directory.
    pub fn outdir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.options.outdir = Some(dir.into());
        self
    }

    /// Set output file (single entry only).
    pub fn outfile(mut self, file: impl Into<PathBuf>) -> Self {
        self.options.outfile = Some(file.into());
        self
    }

    /// Set target platform.
    pub fn platform(mut self, platform: crate::Platform) -> Self {
        self.options.platform = platform;
        self
    }

    /// Set output format.
    pub fn format(mut self, format: crate::OutputFormat) -> Self {
        self.options.format = format;
        self
    }

    /// Enable source maps.
    pub fn sourcemap(mut self, sourcemap: Option<crate::SourceMapType>) -> Self {
        self.options.sourcemap = sourcemap;
        self
    }

    /// Enable minification.
    pub fn minify(mut self, minify: bool) -> Self {
        self.options.minify = minify;
        self
    }

    /// Set global variable names (for IIFE/UMD).
    pub fn globals(mut self, globals: FxHashMap<String, String>) -> Self {
        self.options.globals = globals;
        self
    }

    /// Add a path alias.
    pub fn path_alias(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.options.path_aliases.insert(from.into(), to.into());
        self
    }

    /// Set path aliases.
    pub fn path_aliases(mut self, aliases: FxHashMap<String, String>) -> Self {
        self.options.path_aliases = aliases;
        self
    }

    /// Set current working directory.
    pub fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.options.cwd = Some(cwd.into());
        self
    }

    /// Set runtime for filesystem operations.
    pub fn runtime(mut self, runtime: Arc<dyn Runtime>) -> Self {
        self.options.runtime = Some(runtime);
        self
    }

    /// Execute the full build.
    ///
    /// This performs bundling and returns both the output files and analysis.
    pub async fn build(self) -> Result<BuildResult> {
        execute_build(self.options).await
    }

    /// Convert to an Analyzer for analysis-only (no bundling).
    ///
    /// This allows you to analyze the module graph without generating output files.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use fob_core::builders::unified::builder::Builder;
    ///
    /// # async fn example() -> fob_core::Result<()> {
    /// let analysis = Builder::library("src/index.ts")
    ///     .external(vec!["react"])
    ///     .analyze_only()
    ///     .await?;
    ///
    /// println!("Unused exports: {}", analysis.unused_exports().len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn analyze_only(self) -> Result<crate::analysis::result::AnalysisResult> {
        // Convert BuildOptions to AnalyzerConfig
        let entries = match &self.options.entry {
            EntryPoints::Single(e) => vec![PathBuf::from(e)],
            EntryPoints::Multiple(e) => e.iter().map(|e| PathBuf::from(e)).collect(),
            EntryPoints::Named(e) => e.values().map(|e| PathBuf::from(e)).collect(),
        };

        let mut analyzer = Analyzer::new();
        
        for entry in entries {
            analyzer = analyzer.entry(entry);
        }

        analyzer = analyzer
            .external(self.options.external.clone())
            .path_aliases(self.options.path_aliases.clone());

        if let Some(ref cwd) = self.options.cwd {
            analyzer = analyzer.cwd(cwd.clone());
        }

        if let Some(ref runtime) = self.options.runtime {
            analyzer = analyzer.runtime(Arc::clone(runtime));
        }

        analyzer.analyze().await
    }

    /// Get a reference to the underlying BuildOptions.
    pub fn options(&self) -> &BuildOptions {
        &self.options
    }

    /// Consume the builder and return the BuildOptions.
    pub fn into_options(self) -> BuildOptions {
        self.options
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<BuildOptions> for Builder {
    fn from(options: BuildOptions) -> Self {
        Self { options }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_presets() {
        let library = Builder::library("src/index.ts");
        assert!(!library.options().bundle);

        let app = Builder::app(vec!["src/main.ts"]);
        assert!(app.options().bundle);
        assert!(!app.options().splitting);

        let components = Builder::components(vec!["src/button.ts", "src/badge.ts"]);
        assert!(components.options().bundle);
        assert!(!components.options().splitting);
    }

    #[test]
    fn test_builder_fluent_api() {
        let builder = Builder::new()
            .entry("src/index.ts")
            .bundle(false)
            .external(vec!["react"])
            .minify(true);

        assert!(!builder.options().bundle);
        assert_eq!(builder.options().external.len(), 1);
        assert!(builder.options().minify);
    }
}


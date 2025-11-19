//! Core bundle configuration types shared across Joy crates.

mod css;
mod helpers;
mod html;
mod plugin;
mod transform;
mod types;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

pub use css::{CssOptions, TailwindOptions};
pub use html::HtmlOptions;
pub use plugin::{CacheConfig, PluginBackend, PluginOptions};
pub use transform::{TransformOptions, TypeScriptConfig};
pub use types::{
    EsTarget, ExperimentalOptions, HtmlTemplateType, JsxRuntime, OutputFormat, Platform,
    SourceMapOptions, TypeCheckMode,
};

use helpers::{
    default_output_dir, default_shared_chunk_threshold, default_static_dir, default_true,
};

/// Main bundle configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleOptions {
    /// Entry points (must be ESM modules)
    #[serde(default)]
    pub entries: Vec<PathBuf>,

    /// Directory containing static assets to copy into the output directory
    #[serde(default = "default_static_dir")]
    pub static_dir: Option<PathBuf>,

    /// Output directory for generated chunks
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,

    /// Output format (ESM only for now; future: preserve modules)
    #[serde(default)]
    pub format: OutputFormat,

    /// Platform target
    #[serde(default)]
    pub platform: Platform,

    /// Enable code splitting (dynamic imports become separate chunks)
    #[serde(default = "default_true")]
    pub code_splitting: bool,

    /// Enable minification
    #[serde(default)]
    pub minify: bool,

    /// Source map generation
    #[serde(default)]
    pub source_maps: SourceMapOptions,

    /// Shared chunk threshold (bytes)
    /// If multiple async chunks import the same module and total size > threshold,
    /// extract into shared chunk
    #[serde(default = "default_shared_chunk_threshold")]
    pub shared_chunk_threshold: usize,

    /// External modules (not bundled, treated as runtime imports)
    #[serde(default)]
    pub external: Vec<String>,

    /// Enable experimental features
    #[serde(default)]
    pub experimental: ExperimentalOptions,

    /// Configured plugins
    #[serde(default)]
    pub plugins: Vec<PluginOptions>,

    /// Cache configuration
    #[serde(default)]
    pub cache_config: CacheConfig,

    /// Transform/transpilation options
    #[serde(default)]
    pub transform: TransformOptions,

    /// TypeScript configuration (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typescript_config: Option<TypeScriptConfig>,

    /// HTML generation configuration (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<HtmlOptions>,

    /// Virtual module configuration for build-time code generation
    #[serde(default)]
    pub virtual_modules: Option<VirtualModuleConfig>,

    /// Inline transform functions for project-specific transformations
    #[serde(default)]
    pub inline_transforms: Option<Vec<InlineTransform>>,

    /// CSS processing configuration
    #[serde(default)]
    pub css: CssOptions,

    /// Path aliases for import resolution (e.g., "@components" → "src/components")
    ///
    /// Enables clean imports in MDX and TypeScript files:
    /// ```mdx
    /// import { Hero } from '@components/Hero'
    /// ```
    ///
    /// Key: alias prefix (e.g., "@components")
    /// Value: actual directory path (relative to project root or absolute)
    #[serde(default)]
    pub path_aliases: HashMap<String, PathBuf>,
}

impl BundleOptions {
    /// Create from serde_json::Value (for programmatic config from DB/API)
    ///
    /// # Example
    ///
    /// ```
    /// use fob_config::BundleOptions;
    /// use serde_json::json;
    /// use std::path::PathBuf;
    ///
    /// let value = json!({
    ///     "entries": ["index.mdx"],
    ///     "minify": true,
    ///     "format": "esm"
    /// });
    ///
    /// let options = BundleOptions::from_value(value).unwrap();
    /// assert_eq!(options.entries, vec![PathBuf::from("index.mdx")]);
    /// assert!(options.minify);
    /// ```
    pub fn from_value(value: Value) -> Result<Self, crate::error::ConfigError> {
        serde_json::from_value(value)
            .map_err(|e| crate::error::ConfigError::InvalidValue(e.to_string()))
    }

    /// Convert to serde_json::Value
    ///
    /// # Example
    ///
    /// ```
    /// use fob_config::BundleOptions;
    ///
    /// let options = BundleOptions::default();
    /// let value = options.to_value().unwrap();
    /// ```
    pub fn to_value(&self) -> Result<Value, crate::error::ConfigError> {
        serde_json::to_value(self)
            .map_err(|e| crate::error::ConfigError::InvalidValue(e.to_string()))
    }

    /// Synchronize the top-level minify flag to transform.minify
    ///
    /// This ensures macro transforms have access to the production indicator.
    /// Call this after loading config to ensure consistency.
    pub fn sync_minify(&mut self) {
        self.transform.minify = self.minify;
    }

    /// Add a path alias for import resolution
    ///
    /// # Example
    /// ```
    /// use fob_config::BundleOptions;
    /// use std::path::PathBuf;
    ///
    /// let options = BundleOptions::default()
    ///     .with_alias("@components", "src/components")
    ///     .with_alias("@ui", "src/ui");
    /// ```
    pub fn with_alias(mut self, alias: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        self.path_aliases.insert(alias.into(), path.into());
        self
    }

    /// Add default path aliases relative to a base directory
    ///
    /// Sets up common aliases:
    /// - @components → base_dir/components
    /// - @layouts → base_dir/layouts
    /// - @ui → base_dir/ui
    /// - @utils → base_dir/utils
    ///
    /// # Example
    /// ```
    /// use fob_config::BundleOptions;
    ///
    /// let options = BundleOptions::default()
    ///     .with_default_aliases("src");
    /// ```
    pub fn with_default_aliases(mut self, base_dir: impl AsRef<std::path::Path>) -> Self {
        let base = base_dir.as_ref();
        self.path_aliases
            .insert("@components".to_string(), base.join("components"));
        self.path_aliases
            .insert("@layouts".to_string(), base.join("layouts"));
        self.path_aliases.insert("@ui".to_string(), base.join("ui"));
        self.path_aliases
            .insert("@utils".to_string(), base.join("utils"));
        self
    }
}

impl Default for BundleOptions {
    fn default() -> Self {
        Self {
            entries: vec![],
            static_dir: default_static_dir(),
            output_dir: PathBuf::from("dist"),
            format: OutputFormat::Esm,
            platform: Platform::Browser,
            code_splitting: true,
            minify: false,
            source_maps: SourceMapOptions::default(),
            shared_chunk_threshold: 20_000, // 20KB
            external: vec![],
            experimental: ExperimentalOptions::default(),
            plugins: Vec::new(),
            cache_config: CacheConfig::default(),
            transform: TransformOptions::default(),
            typescript_config: None,
            html: None,
            virtual_modules: None,
            inline_transforms: None,
            css: CssOptions::default(),
            path_aliases: HashMap::new(),
        }
    }
}

/// Configuration for virtual modules
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VirtualModuleConfig {
    /// Map of virtual module ID to JavaScript generator function code
    /// The generator function should return the module source code as a string
    ///
    /// Example:
    /// ```json
    /// {
    ///   "virtual:api-data": "async function() { return 'export default {}'; }"
    /// }
    /// ```
    #[serde(default)]
    pub modules: HashMap<String, String>,
}

/// Configuration for an inline transform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineTransform {
    /// Glob pattern or regex to match files
    pub test: String,

    /// JavaScript transform function code
    /// Function signature: (code: string, context: { path: string }) => string
    pub transform: String,
}

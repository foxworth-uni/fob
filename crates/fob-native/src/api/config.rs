//! Bundle configuration types

use crate::types::OutputFormat;
use napi_derive::napi;

/// MDX compilation options
///
/// Configure how MDX files are compiled to JSX. All options are optional
/// and have sensible defaults. MDX is auto-enabled when .mdx entries are detected.
#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct MdxOptions {
    /// Enable GitHub Flavored Markdown (tables, strikethrough, task lists)
    /// Default: true
    pub gfm: Option<bool>,
    /// Enable footnotes
    /// Default: true
    pub footnotes: Option<bool>,
    /// Enable math support ($inline$ and $$block$$)
    /// Default: true
    pub math: Option<bool>,
    /// JSX runtime module path
    /// Default: "react/jsx-runtime"
    pub jsx_runtime: Option<String>,
    /// Use default plugins (heading IDs, image optimization)
    /// Default: true
    pub use_default_plugins: Option<bool>,
}

/// Bundle configuration
#[napi(object)]
pub struct BundleConfig {
    /// Entry points to bundle
    pub entries: Vec<String>,
    /// Output directory (defaults to "dist" if not provided)
    pub output_dir: Option<String>,
    /// Output format ("esm" | "cjs" | "iife")
    pub format: Option<OutputFormat>,
    /// Source map generation mode
    /// Can be a boolean string ("true" = external, "false" = disabled) or a mode string ("inline" | "hidden" | "external")
    /// In JavaScript/TypeScript, pass as string: "true", "false", "inline", "hidden", or "external"
    pub sourcemap: Option<String>,
    /// Packages to externalize (not bundled)
    pub external: Option<Vec<String>>,
    /// Target platform ("browser" | "node", default: "browser")
    pub platform: Option<String>,
    /// Enable minification (default: false)
    pub minify: Option<bool>,
    /// Working directory for resolution
    pub cwd: Option<String>,
    /// MDX compilation options
    /// If None, MDX is auto-enabled when .mdx entries are detected
    pub mdx: Option<MdxOptions>,
}

// Note: Builder pattern is not exposed via NAPI due to limitations with moving self.
// Users can construct BundleConfig directly in JavaScript/TypeScript.

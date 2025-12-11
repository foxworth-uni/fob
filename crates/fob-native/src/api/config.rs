//! Bundle configuration types

use crate::api::primitives::CodeSplittingConfig;
use napi_derive::napi;
use std::collections::HashMap;

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
#[derive(Debug, Clone, Default)]
pub struct BundleConfig {
    /// Entry points to bundle
    pub entries: Vec<String>,
    /// Output directory (defaults to "dist" if not provided)
    pub output_dir: Option<String>,
    /// Output format: "esm" | "cjs" | "iife" (case-insensitive, default: "esm")
    pub format: Option<String>,
    /// Source map generation mode
    /// Accepts: "true", "false", "external", "inline", "hidden"
    /// Default: disabled (no source maps)
    pub sourcemap: Option<String>,
    /// Target platform ("browser" | "node", default: "browser")
    pub platform: Option<String>,
    /// Enable minification (default: false)
    pub minify: Option<bool>,
    /// Working directory for resolution
    pub cwd: Option<String>,
    /// MDX compilation options
    /// If None, MDX is auto-enabled when .mdx entries are detected
    pub mdx: Option<MdxOptions>,

    // === Composable Primitives ===
    /// Entry mode: "shared" | "isolated" (case-insensitive, default: "shared")
    /// - shared: Entries can share chunks
    /// - isolated: Each entry stands alone
    pub entry_mode: Option<String>,
    /// Code splitting configuration
    /// - None/undefined: No code splitting
    /// - Object: Enable code splitting with specified thresholds
    pub code_splitting: Option<CodeSplittingConfig>,
    /// External packages (simplified for JS)
    /// - None/undefined: Bundle everything
    /// - Array: Externalize specific packages
    pub external: Option<Vec<String>>,
    /// Externalize dependencies from package.json manifest
    /// - true: Read dependencies/peerDependencies from package.json
    /// - false/undefined: Use explicit external list or bundle all
    pub external_from_manifest: Option<bool>,

    /// Virtual files mapping (path → content)
    /// Used internally when entries have inline content via the JS wrapper.
    /// Keys should use "virtual:" prefix (e.g., "virtual:main.ts")
    pub virtual_files: Option<HashMap<String, String>>,
}

// Note: Builder pattern is not exposed via NAPI due to limitations with moving self.
// Users can construct BundleConfig directly in JavaScript/TypeScript.

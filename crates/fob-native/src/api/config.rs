//! Bundle configuration types

use crate::types::OutputFormat;
use napi_derive::napi;

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
}

// Note: Builder pattern is not exposed via NAPI due to limitations with moving self.
// Users can construct BundleConfig directly in JavaScript/TypeScript.

//! Bundle configuration types

use crate::types::{OutputFormat, SourceMapMode};
use napi_derive::napi;

/// Bundle configuration
#[napi(object)]
pub struct BundleConfig {
    /// Entry points to bundle
    pub entries: Vec<String>,
    /// Output directory (defaults to "dist" if not provided)
    pub output_dir: Option<String>,
    /// Output format
    pub format: Option<OutputFormat>,
    /// Source map generation mode
    pub sourcemap: Option<SourceMapMode>,
    /// Working directory for resolution
    pub cwd: Option<String>,
}

// Note: Builder pattern is not exposed via NAPI due to limitations with moving self.
// Users can construct BundleConfig directly in JavaScript/TypeScript.

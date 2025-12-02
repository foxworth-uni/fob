use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::bundle::helpers::{default_mode, default_true};
use crate::bundle::types::{EsTarget, TypeCheckMode};

/// Transformation/transpilation options for TypeScript and JSX
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformOptions {
    /// Enable TypeScript to JavaScript transformation
    #[serde(default = "default_true")]
    pub typescript: bool,

    /// Enable JSX to JavaScript transformation
    #[serde(default = "default_true")]
    pub jsx: bool,

    /// Target ECMAScript version
    #[serde(default)]
    pub target: EsTarget,

    /// Type-checking mode (currently experimental)
    #[serde(default)]
    pub type_check: TypeCheckMode,

    /// JSX import source for automatic runtime (e.g., "@emotion/react", "solid-js")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsx_import_source: Option<String>,

    /// Use development mode for JSX (jsxDEV instead of jsx)
    #[serde(default = "default_true")]
    pub jsx_dev: bool,

    /// Define replacements for compile-time constant folding
    ///
    /// Maps identifiers to replacement values (as JSON strings).
    /// Example: `{"process.env.NODE_ENV": "\"production\""}`
    ///
    /// # Security
    /// For browser platform, only `process.env.NODE_ENV` is allowed by default
    /// to prevent accidental exposure of server-side environment variables.
    ///
    /// # Dead Code Elimination
    /// Define replacements enable DCE by replacing conditionals like
    /// `if (process.env.NODE_ENV === 'production')` with `if ("production" === 'production')`,
    /// which minifiers can then optimize away.
    #[serde(default)]
    pub define: HashMap<String, String>,

    /// Enable SSR module transform (Vite-style module loading)
    ///
    /// When enabled, transforms ES modules into SSR-compatible code with runtime
    /// module resolution. This enables:
    /// - Dynamic module loading via `__vite_ssr_import__`
    /// - Hot module replacement support
    /// - Better async import/export handling
    ///
    /// Only used when platform is Node or for SSR builds.
    #[serde(default)]
    pub enable_ssr_transform: bool,

    /// Build mode for environment-specific code transformations
    /// Typically "development" or "production"
    #[serde(default = "default_mode")]
    pub mode: String,

    /// Public environment variables (filtered for browser safety)
    /// Only variables with PUBLIC_ prefix should be included
    #[serde(default)]
    pub public_env: HashMap<String, String>,

    /// Enable minification (production indicator for macro transforms)
    ///
    /// This flag serves dual purposes:
    /// 1. Controls code minification in the output phase
    /// 2. Indicates production builds to macro transforms (@dev removes code when minify=true)
    ///
    /// Typically true in production, false in development.
    #[serde(default)]
    pub minify: bool,
}

impl Default for TransformOptions {
    fn default() -> Self {
        Self {
            typescript: true,
            jsx: true,
            target: EsTarget::default(),
            type_check: TypeCheckMode::default(),
            jsx_import_source: None,
            jsx_dev: true,
            define: HashMap::new(),
            enable_ssr_transform: false, // Opt-in for SSR builds only
            mode: "development".to_string(),
            public_env: HashMap::new(),
            minify: false, // Development default
        }
    }
}

/// TypeScript configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeScriptConfig {
    /// Path to tsconfig.json (None = auto-discovery disabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<PathBuf>,

    /// Override jsxImportSource from tsconfig (e.g., "@emotion/react", "solid-js")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsx_import_source: Option<String>,

    /// Allow .js files to import .ts files (default: true)
    #[serde(default = "default_true")]
    pub allow_js: bool,

    /// Generate TypeScript declaration files (.d.ts)
    /// When None, auto-detects based on entry file extensions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emit_declarations: Option<bool>,

    /// Directory for emitting declaration files (relative to output_dir)
    /// If None, .d.ts files are emitted alongside .js files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration_dir: Option<PathBuf>,

    /// Strip @internal declarations from .d.ts files (default: false)
    #[serde(default)]
    pub strip_internal: bool,

    /// Generate declaration source maps (.d.ts.map) (default: false)
    #[serde(default)]
    pub declaration_map: bool,
}

impl Default for TypeScriptConfig {
    fn default() -> Self {
        Self {
            config_path: None,
            jsx_import_source: None,
            allow_js: true,
            emit_declarations: None,
            declaration_dir: None,
            strip_internal: false,
            declaration_map: false,
        }
    }
}

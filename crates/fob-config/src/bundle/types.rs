use serde::{Deserialize, Serialize};

/// Output format for bundles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Standard ESM bundle with runtime
    Esm,
    /// Preserve module structure (experimental)
    #[serde(rename = "preserve-modules")]
    PreserveModules,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Esm
    }
}

/// Target platform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    /// Browser environment (default)
    Browser,
    /// Node.js (ESM mode only)
    Node,
    /// Web Workers
    Worker,
    /// Deno runtime
    Deno,
}

impl Default for Platform {
    fn default() -> Self {
        Self::Browser
    }
}

/// Source map generation options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceMapOptions {
    /// No source maps
    None,
    /// Inline source maps (base64)
    Inline,
    /// External .map files
    External,
    /// External with source content embedded
    ExternalWithContent,
}

impl Default for SourceMapOptions {
    fn default() -> Self {
        Self::External
    }
}

/// Experimental features (unstable APIs)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExperimentalOptions {
    /// Enable WASM module support (import assertions)
    #[serde(default)]
    pub wasm: bool,

    /// Enable CSS module support (via plugin)
    #[serde(default)]
    pub css: bool,

    /// Enable JSON module support
    #[serde(default = "crate::bundle::helpers::default_true")]
    pub json: bool,

    /// Emit bundle analysis data
    #[serde(default)]
    pub analysis: bool,
}

/// Target ECMAScript version for transpilation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EsTarget {
    /// ECMAScript 2015 (ES6)
    ES2015,
    /// ECMAScript 2016
    ES2016,
    /// ECMAScript 2017
    ES2017,
    /// ECMAScript 2018
    ES2018,
    /// ECMAScript 2019
    ES2019,
    /// ECMAScript 2020
    ES2020,
    /// ECMAScript 2021
    ES2021,
    /// ECMAScript 2022 (default)
    ES2022,
    /// ECMAScript 2023
    ES2023,
    /// ECMAScript 2024
    ES2024,
    /// Latest ECMAScript (currently ES2024)
    ESNext,
}

impl Default for EsTarget {
    fn default() -> Self {
        Self::ES2022
    }
}

/// Type-checking mode for TypeScript
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TypeCheckMode {
    /// No type-checking (transpile-only)
    None,
    // Future: Semantic type-checking (experimental, may use oxc_isolated_declarations or tsc)
    // Semantic,
}

impl Default for TypeCheckMode {
    fn default() -> Self {
        Self::None
    }
}

/// JSX transformation runtime mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JsxRuntime {
    /// Classic runtime: React.createElement (legacy)
    Classic,
    /// Automatic runtime: react/jsx-runtime (modern, default)
    Automatic,
}

impl Default for JsxRuntime {
    fn default() -> Self {
        Self::Automatic
    }
}

/// Type of built-in HTML template to use
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HtmlTemplateType {
    /// Single-page application template (default)
    Spa,
    /// Multi-page application template
    Mpa,
}

impl Default for HtmlTemplateType {
    fn default() -> Self {
        Self::Spa
    }
}


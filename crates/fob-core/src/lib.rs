#![cfg_attr(docsrs, feature(doc_cfg))]

//! # joy-core
//!
//! Joy’s task-based API for bundling and analysis on top of Rollup-compatible
//! [Rolldown](https://rolldown.rs).
//!
//! ## Quick Start
//!
//! ### Bundle a library
//!
//! ```no_run
//! use fob_core as fob;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let result = fob::BuildOptions::library("./src/index.js")
//!     .externals(["react", "react-dom"])
//!     .sourcemap(true)
//!     .build()
//!     .await?;
//!
//! let bundle = result.output.as_single().expect("single bundle");
//! for asset in bundle.assets.iter() {
//!     std::fs::write(format!("dist/{}", asset.filename()), asset.content_as_bytes())?;
//! }
//! # Ok(()) }
//! ```
//!
//! ### Bundle component islands
//!
//! ```no_run
//! use fob_core as fob;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let result = fob::BuildOptions::components([
//!     "./src/button.js",
//!     "./src/badge.js",
//! ])
//! .build()
//! .await?;
//!
//! let bundles = result.output.as_multiple().expect("multiple bundles");
//! for (name, bundle) in bundles.iter() {
//!     println!("bundle {name} emits {} assets", bundle.assets.len());
//! }
//! # Ok(()) }
//! ```
//!
//! ### Bundle a multi-entry application
//!
//! ```no_run
//! use fob_core as fob;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let result = fob::BuildOptions::app(["./src/main.js", "./src/dashboard.js"])
//!     .build()
//!     .await?;
//!
//! let bundle = result.output.as_single().expect("single bundle");
//! for asset in bundle.assets.iter() {
//!     println!("emitted {}", asset.filename());
//! }
//! # Ok(()) }
//! ```
//!
//! ### Analyze without bundling
//!
//! ```no_run
//! use fob_core as fob;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let analysis = fob::analyze(["./src/index.js"]).await?;
//! for unused in analysis.graph.unused_exports() {
//!     println!("unused: {} from {}", unused.export.name, unused.module_id);
//! }
//! # Ok(()) }
//! ```
//!
//! ### Generate TypeScript declarations
//!
//! ```no_run
//! use fob_core as fob;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Auto-detects TypeScript and generates .d.ts files
//! let result = fob::BuildOptions::library("./src/index.ts")
//!     .build()
//!     .await?;
//!
//! // Or with explicit configuration
//! let result = fob::BuildOptions::library("./src/lib.ts")
//!     .emit_dts(true)            // Explicitly enable
//!     .strip_internal(true)      // Remove @internal declarations
//!     .dts_dir("types")          // Custom output directory
//!     .declaration_map(true)     // Generate .d.ts.map
//!     .build()
//!     .await?;
//!
//! let bundle = result.output.as_single().expect("single bundle");
//! for asset in bundle.assets.iter() {
//!     if asset.filename().ends_with(".d.ts") {
//!         println!("Generated: {}", asset.filename());
//!     }
//! }
//! # Ok(()) }
//! ```

#[cfg(feature = "rolldown-integration")]
pub mod analysis;
pub mod builders;
pub mod graph;
pub mod output;
pub mod runtime;

// Re-export primary APIs
#[cfg(feature = "rolldown-integration")]
pub use analysis::analyzer::Analyzer;
// Note: Builder is not publicly exported from unified module
#[cfg(feature = "rolldown-integration")]
pub use analysis::{analyze, analyze_with_options};

// Test utilities (available in test builds for both unit and integration tests)
// The cfg(any(test, doctest)) makes this available when:
// - Running unit tests (cfg(test))
// - Running integration tests (cfg(test) on the integration test binary)
// - Running doc tests (cfg(doctest))
// We also gate on not(target_family = "wasm") since TestRuntime uses std::fs
#[cfg(all(any(test, doctest), not(target_family = "wasm")))]
pub mod test_utils;

// Platform-specific runtime implementations
#[cfg(not(target_family = "wasm"))]
pub mod native_runtime;
#[cfg(not(target_family = "wasm"))]
pub use native_runtime::NativeRuntime;

#[cfg(target_family = "wasm")]
pub mod wasm_runtime;
#[cfg(target_family = "wasm")]
pub use wasm_runtime::WasmRuntime;

#[cfg(feature = "dts-generation")]
pub mod plugins;

// Re-export core Rolldown types for library users
pub use rolldown::{
    BundleOutput, Bundler, BundlerOptions, InputItem, OutputFormat, Platform, SourceMapType,
};

// Re-export output types for detailed bundle inspection
pub use rolldown_common::{Output, OutputAsset, OutputChunk};

// Re-export TypeScript-related types from rolldown_common
#[cfg(feature = "rolldown-integration")]
pub use rolldown_common::{
    BundlerTransformOptions, IsolatedDeclarationsOptions, TypeScriptOptions,
};

// Re-export runtime types
pub use runtime::{FileMetadata, Runtime, RuntimeError, RuntimeResult};

pub use graph::{
    Export, ExportKind, Import, ImportKind, ImportSpecifier, Module, ModuleId, ModuleIdError,
    SourceSpan, SourceType,
};

#[cfg(feature = "rolldown-integration")]
pub use analysis::{
    AnalysisResult, AnalyzeError, AnalyzedBundle, CacheAnalysis, CacheEffectiveness,
    ImportOutcome, ImportResolution, RenameEvent, RenamePhase, TransformationTrace,
};

pub use builders::{build, plugin, BuildOptions, BuildOutput, BuildResult, EntryPoints};

#[cfg(feature = "dts-generation")]
pub use builders::DtsOptions;

// Re-export DtsEmitPlugin from plugins module when dts-generation feature is enabled
#[cfg(feature = "dts-generation")]
#[cfg_attr(docsrs, doc(cfg(feature = "dts-generation")))]
pub use plugins::DtsEmitPlugin;

#[cfg(feature = "docs-generation")]
#[cfg_attr(docsrs, doc(cfg(feature = "docs-generation")))]
pub use fob_docs::{
    DocsEmitPlugin, DocsEmitPluginOptions, DocsPluginOutputFormat, Documentation, ExportedSymbol,
    JsDocTag, ModuleDoc, ParameterDoc, SymbolKind,
};

#[cfg(feature = "llm-docs")]
#[cfg_attr(docsrs, doc(cfg(feature = "llm-docs")))]
pub use fob_docs::llm::{EnhancementMode, LlmConfig, LlmEnhancer};

pub use output::{AppBuild, Bundle as JoyBundle, ComponentBuild, ImportMap};

pub use fob_plugin_mdx::BunnyMdxPlugin;
pub use rolldown_plugin::{Plugin, __inner::SharedPluginable};

/// Error types for joy-core operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error from Rolldown bundler.
    #[error("Rolldown bundler error: {0}")]
    Bundler(String),

    /// Invalid configuration provided.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid output path (e.g., directory traversal attempt).
    #[error("Invalid output path: {0}")]
    InvalidOutputPath(String),

    /// File write operation failed.
    #[error("Write failure: {0}")]
    WriteFailure(String),

    /// Output file already exists and overwrite is disabled.
    #[error("Output exists: {0}")]
    OutputExists(String),

    /// Asset not found during resolution.
    #[error("Asset not found: {specifier} (searched from: {searched_from})")]
    AssetNotFound {
        specifier: String,
        searched_from: String,
    },

    /// Asset security violation (e.g., directory traversal attempt).
    #[error("Asset security violation: {path} - {reason}")]
    AssetSecurityViolation {
        path: String,
        reason: String,
    },

    /// Asset file is too large.
    #[error("Asset too large: {path} ({size} bytes exceeds limit of {max_size} bytes)")]
    AssetTooLarge {
        path: String,
        size: u64,
        max_size: u64,
    },

    /// I/O error with context message.
    #[error("{message}")]
    IoError {
        message: String,
        #[source]
        source: std::io::Error,
    },
}

/// Result type alias for fob-core operations.
pub type Result<T> = std::result::Result<T, Error>;

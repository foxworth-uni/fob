#![cfg_attr(docsrs, feature(doc_cfg))]

//! # fob
//!
//! Fob foundation crate - Graph analysis and runtime abstraction.
//!
//! This crate provides the core graph analysis capabilities, runtime abstraction,
//! and module graph primitives. It's designed to be lightweight and WASM-compatible,
//! making it suitable for IDE integrations, analysis tools, and other applications
//! that don't need full bundling capabilities.
//!
//! ## Features
//!
//! - **Graph Analysis**: Module dependency graph with import/export tracking
//! - **Standalone Analyzer**: Fast analysis without bundling overhead
//! - **Runtime Abstraction**: Platform-agnostic filesystem operations
//! - **WASM Compatible**: Works in browser and WASI environments
//!
//! ## Quick Start
//!
//! ### Analyze without bundling
//!
//! ```no_run
//! use fob::analyze;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let analysis = analyze(["./src/index.js"]).await?;
//! for unused in analysis.graph.unused_exports()? {
//!     println!("unused: {} from {}", unused.export.name, unused.module_id);
//! }
//! # Ok(()) }
//! ```
//!
//! ### Use the Analyzer API
//!
//! ```no_run
//! use fob::Analyzer;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let analysis = Analyzer::new()
//!     .entry("./src/index.ts")
//!     .external(vec!["react", "lodash"])
//!     .path_alias("@", "./src")
//!     .analyze()
//!     .await?;
//!
//! println!("Unused exports: {}", analysis.unused_exports()?.len());
//! # Ok(()) }
//! ```

pub mod analysis;
pub mod extractors;
pub mod graph;
pub mod runtime;

// Re-export primary APIs
pub use analysis::analyzer::Analyzer;
pub use analysis::{analyze, analyze_with_options};

// Test utilities (available in test builds and when test-utils feature is enabled)
// Note: test_utils requires tokio, so it's only available on native platforms
#[cfg(any(
    all(any(test, doctest), not(target_family = "wasm")),
    all(feature = "test-utils", not(target_family = "wasm"))
))]
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

// Re-export runtime types
pub use runtime::{FileMetadata, Runtime, RuntimeError, RuntimeResult};

// Re-export graph types
pub use graph::{
    Export, ExportKind, Import, ImportKind, ImportSpecifier, Module, ModuleGraph, ModuleId,
    ModuleIdError, SourceSpan, SourceType,
};

// Re-export analysis types
pub use analysis::{
    AnalysisResult, AnalyzeError, CacheAnalysis, CacheEffectiveness, ImportOutcome,
    ImportResolution, RenameEvent, RenamePhase, TransformationTrace,
};

// Re-export MDX plugin (WASM-compatible)
pub use fob_plugin_mdx::BunnyMdxPlugin;

// Re-export OXC foundation types for consistent version usage across workspace
// These are commonly used types that appear in public APIs and cross crate boundaries
pub mod oxc {
    //! OXC (Oxidation Compiler) foundation types re-exported for workspace consistency.
    //!
    //! This ensures all workspace crates use the same OXC version for types that
    //! cross crate boundaries. Specialized OXC crates (like `oxc_isolated_declarations`)
    //! should still be imported directly by crates that need them.

    /// Re-export allocator - required for all OXC AST operations
    pub use oxc_allocator::Allocator;

    /// Re-export AST types
    pub use oxc_ast::ast;

    /// Re-export AST visitor trait
    pub use oxc_ast_visit::Visit;

    /// Re-export span types for source location tracking
    pub use oxc_span::{GetSpan, SourceType, Span};

    /// Re-export parser for code analysis
    pub use oxc_parser::{Parser, ParserReturn};

    /// Re-export semantic analysis
    pub use oxc_semantic::{ScopeFlags, SemanticBuilder};
}

// Note: AnalyzedBundle is available in fob-bundler, not here

/// Error types for fob operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid configuration provided.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Analysis or graph operation error.
    #[error("Operation error: {0}")]
    Operation(String),
}

/// Result type alias for fob operations.
pub type Result<T> = std::result::Result<T, Error>;

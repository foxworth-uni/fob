#![cfg_attr(docsrs, feature(doc_cfg))]

//! # fob
//!
//! Fob foundation crate - Runtime abstraction and core types.
//!
//! This crate provides the runtime abstraction layer. For graph and analysis
//! functionality, use the `fob-graph` and `fob-analysis` crates respectively.

pub mod runtime;

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

// Re-export MDX plugin (WASM-compatible)
// pub use fob_plugin_mdx::BunnyMdxPlugin;  // Moved to bunny repository

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

// Re-export core crates that don't depend on fob (to avoid cycles)
// Note: fob-graph, fob-analysis, and fob-bundler depend on fob, so they cannot be re-exported
#[cfg(not(target_family = "wasm"))]
pub use fob_gen as codegen;
#[cfg(not(target_family = "wasm"))]
pub use fob_config as config;

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

//! # fob-graph
//!
//! Pure graph data structures for module dependency graphs.
//!
//! This crate provides the core graph primitives and `ModuleGraph` implementation
//! without any I/O or analysis logic. It's designed to be lightweight and
//! WASM-compatible.
//!
//! ## Overview
//!
//! `fob-graph` is the foundation for building module dependency graphs from
//! JavaScript/TypeScript codebases. It provides:
//!
//! - **Pure Data Structures**: No I/O, no file system dependencies
//! - **WASM-Compatible**: Can run in browser environments
//! - **Thread-Safe**: Uses `Arc` for efficient shared ownership
//! - **Memory Efficient**: Arena-based allocation where possible
//! - **Type-Safe**: Strong typing for modules, imports, exports, and dependencies
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    ModuleGraph                              │
//! │  (Arc-based, thread-safe, WASM-compatible)                  │
//! └────────────────────┬────────────────────────────────────────┘
//!                      │
//!          ┌───────────┼───────────┐
//!          │           │           │
//!          ▼           ▼           ▼
//!    ┌─────────┐ ┌─────────┐ ┌─────────┐
//!    │ Module  │ │ Import  │ │ Export  │
//!    │ (Node)  │ │ (Edge)  │ │ (Edge)  │
//!    └─────────┘ └─────────┘ └─────────┘
//!          │           │           │
//!          └───────────┼───────────┘
//!                      │
//!                      ▼
//!          ┌──────────────────────┐
//!          │   SymbolTable        │
//!          │   (Intra-file        │
//!          │    analysis)         │
//!          └──────────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ### Building a Module Graph
//!
//! ```rust,no_run
//! use fob_graph::{ModuleGraph, Module, ModuleId, SourceType};
//! use std::path::PathBuf;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new graph
//! let graph = ModuleGraph::new()?;
//!
//! // Add a module
//! let module_id = ModuleId::new("src/index.ts")?;
//! let module = Module::builder(module_id.clone(), PathBuf::from("src/index.ts"), SourceType::TypeScript)
//!     .entry(true)
//!     .build();
//!
//! graph.add_module(module)?;
//!
//! // Query the graph
//! let dependencies = graph.dependencies(&module_id)?;
//! println!("Dependencies: {:?}", dependencies);
//! # Ok(())
//! # }
//! ```
//!
//! ### Symbol Analysis
//!
//! ```rust,no_run
//! use fob_graph::semantic::analyze_symbols;
//! use fob_graph::{SourceType, SymbolTable};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let source = r#"
//!     const used = 42;
//!     const unused = 100;
//!     console.log(used);
//! "#;
//!
//! let table: SymbolTable = analyze_symbols(source, "example.js", SourceType::JavaScript)?;
//!
//! // Find unused symbols
//! let unused = table.unused_symbols();
//! println!("Unused symbols: {:?}", unused);
//! # Ok(())
//! # }
//! ```
//!
//! ## Extension Trait Pattern
//!
//! The crate uses extension traits to add functionality without modifying core types.
//! Methods are organized into logical groups:
//!
//! - [`memory::queries`] - Query operations (dependencies, dependents, etc.)
//! - [`memory::mutations`] - Modification operations (add module, add dependency)
//! - [`memory::exports`] - Export analysis (unused exports, usage counts)
//! - [`memory::symbols`] - Symbol-level analysis (unused symbols, statistics)
//! - [`memory::chains`] - Dependency chain analysis (circular detection)
//!
//! ## Thread Safety
//!
//! `ModuleGraph` uses `Arc` internally for efficient shared ownership. Multiple threads
//! can safely read from the graph concurrently. For modifications, use appropriate
//! synchronization (e.g., `Mutex` or `RwLock`).
//!
//! ## WASM Compatibility
//!
//! The crate is designed to work in WASM environments:
//! - No file system dependencies
//! - No network dependencies
//! - Pure Rust data structures
//! - Compatible with `wasm-bindgen` and `wasm-pack`

// Runtime abstraction (merged from fob-core)
pub mod runtime;

// Analysis functionality (merged from fob-analysis)
#[path = "analysis/lib.rs"]
pub mod analysis;

// Core graph types and primitives
mod class_enum_extraction;
pub mod collection;
pub mod dependency_chain;
pub mod export;
pub mod external_dep;
pub mod framework_rules;
pub mod from_collection;
pub mod import;
pub mod module;
pub mod module_id;
pub mod package_json;
mod quality;
pub mod semantic;
pub mod span;
pub mod statistics;
pub mod symbol;

// ModuleGraph implementation
// In-memory implementation (WASM-compatible)
mod memory;

// Re-export ModuleGraph implementation
pub use memory::{
    ClassMemberInfo, EnumMemberInfo, ModuleGraph, NamespaceImportInfo, SideEffectImport,
    TypeOnlyImport,
};

/// Output entry for unused exports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnusedExport {
    pub module_id: module_id::ModuleId,
    pub export: export::Export,
}

pub use collection::{
    CollectedExport, CollectedImport, CollectedImportSpecifier, CollectedModule, CollectionState,
    parse_module_structure,
};
pub use dependency_chain::{ChainAnalysis, DependencyChain};
pub use export::{Export, ExportKind};
pub use external_dep::ExternalDependency;
pub use framework_rules::FrameworkRule;
pub use from_collection::CollectionGraphError;
pub use import::{Import, ImportKind, ImportSpecifier};
pub use module::{Module, SourceType};
pub use module_id::{ModuleId, ModuleIdError};
pub use package_json::{
    DependencyCoverage, DependencyType, PackageJson, TypeCoverage, UnusedDependency,
    extract_package_name,
};
pub use span::SourceSpan;
pub use statistics::GraphStatistics;
pub use symbol::{
    ClassMemberMetadata, EnumMemberMetadata, EnumMemberValue, Symbol, SymbolKind, SymbolMetadata,
    SymbolSpan, SymbolStatistics, SymbolTable, UnreachableCode, UnusedSymbol, Visibility,
};

// Re-export runtime types
pub use runtime::{FileMetadata, Runtime, RuntimeError, RuntimeResult};

// Platform-specific runtime implementations
#[cfg(not(target_family = "wasm"))]
pub use runtime::native::NativeRuntime;

#[cfg(target_family = "wasm")]
pub use runtime::wasm::WasmRuntime;

// Test utilities (available in test builds)
#[cfg(any(
    all(any(test, doctest), not(target_family = "wasm")),
    all(feature = "test-utils", not(target_family = "wasm"))
))]
pub use runtime::test_utils::TestRuntime;

#[cfg(any(
    all(any(test, doctest), not(target_family = "wasm")),
    all(feature = "test-utils", not(target_family = "wasm"))
))]
pub mod test_utils {
    pub use super::runtime::test_utils::*;
}

// Re-export analysis types (merged from fob-analysis)
pub use analysis::{
    AnalysisResult, AnalyzeError, AnalyzeOptions, Analyzer, CacheAnalysis, CacheEffectiveness,
    Configured, ImportOutcome, ImportResolution, RenameEvent, RenamePhase, TransformationTrace,
    Unconfigured, analyze, analyze_with_options,
};

// Re-export OXC foundation types for consistent version usage across workspace
// These are commonly used types that appear in public APIs and cross crate boundaries
pub mod oxc {
    //! OXC (Oxidation Compiler) foundation types re-exported for workspace consistency.
    //!
    //! This ensures all workspace crates use the same OXC version for types that
    //! cross crate boundaries. Upstream consumers can use `fob_bundler::oxc::*`
    //! instead of importing oxc crates directly.
    //!
    //! # Example
    //!
    //! ```ignore
    //! use fob_bundler::oxc::{Allocator, Parser, SourceType, Codegen};
    //!
    //! let allocator = Allocator::default();
    //! let source = "const x = 1;";
    //! let ret = Parser::new(&allocator, source, SourceType::mjs()).parse();
    //! ```

    // =========================================================================
    // Core: Allocator & Spans
    // =========================================================================

    /// Arena allocator for AST nodes
    pub use oxc_allocator::Allocator;

    /// Span types for source location tracking
    pub use oxc_span::{CompactStr, GetSpan, SourceType, Span};

    // =========================================================================
    // Parsing & AST
    // =========================================================================

    /// AST node types
    pub use oxc_ast::ast;

    /// AST visitor trait
    pub use oxc_ast_visit::Visit;

    /// JavaScript/TypeScript parser
    pub use oxc_parser::{Parser, ParserReturn};

    // =========================================================================
    // Semantic Analysis
    // =========================================================================

    /// Semantic analysis (scopes, symbols, references)
    pub use oxc_semantic::{
        ScopeFlags, Semantic, SemanticBuilder, SemanticBuilderReturn, SymbolFlags,
    };

    // =========================================================================
    // Code Generation
    // =========================================================================

    /// Code generator (AST to string)
    pub use oxc_codegen::{Codegen, CodegenOptions, CodegenReturn};

    // =========================================================================
    // Minification
    // =========================================================================

    /// JavaScript minifier
    pub use oxc_minifier::{Minifier, MinifierOptions, MinifierReturn};

    // =========================================================================
    // Transformation
    // =========================================================================

    /// AST transformer (JSX, TypeScript, etc.)
    pub use oxc_transformer::{TransformOptions, Transformer};

    /// AST traversal utilities
    pub use oxc_traverse::{Traverse, TraverseCtx};

    // =========================================================================
    // TypeScript Declarations
    // =========================================================================

    /// Isolated declarations (.d.ts generation)
    pub use oxc_isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions};
}

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

#[cfg(test)]
mod tests;

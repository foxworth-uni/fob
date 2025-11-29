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

#[cfg(test)]
mod tests;

//! Core graph primitives shared across Joy's analyzers and builders.
//!
//! These types form the foundation for the ModuleGraph implementation described in
//! `design/02-graph-architecture.md`. They intentionally keep domain logic light so
//! higher-level phases can compose them without pulling in heavy dependencies.
//!
//! ```rust
//! use fob::graph::{
//!     analyze_entries, Import, ImportKind, ImportSpecifier, Module, ModuleGraph, ModuleId,
//!     SourceSpan, SourceType,
//! };
//!
//! let utils_id = ModuleId::new_virtual("virtual:utils.ts");
//! let utils = Module::builder(
//!     utils_id.clone(),
//!     "virtual:utils.ts".into(),
//!     SourceType::TypeScript,
//! )
//! .build();
//!
//! let entry_id = ModuleId::new_virtual("virtual:entry.ts");
//! let mut entry = Module::builder(
//!     entry_id.clone(),
//!     "virtual:entry.ts".into(),
//!     SourceType::TypeScript,
//! )
//! .imports(vec![Import::new(
//!     "virtual:utils.ts",
//!     vec![ImportSpecifier::Named("format".into())],
//!     ImportKind::Static,
//!     Some(utils_id.clone()),
//!     SourceSpan::new("virtual:entry.ts", 0, 0),
//! )])
//! .build();
//! entry.mark_entry();
//!
//! let mut graph = ModuleGraph::new();
//! graph.add_module(utils);
//! graph.add_module(entry);
//! graph.add_dependency(entry_id, utils_id);
//!
//! assert!(graph.unused_exports().is_empty());
//! ```
//!
//! ```no_run
//! use fob::graph::analyze_entries;
//! use std::path::PathBuf;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let project_root = PathBuf::from("./examples/my-app");
//! let graph = analyze_entries(&[project_root.join("src/index.js")], &project_root).await?;
//! println!("modules: {}", graph.len());
//! # Ok(())
//! # }
//! ```
//!

// Core graph types and primitives
pub mod collection;
pub mod export;
pub mod external_dep;
pub mod framework_rules;
pub mod from_collection;
pub mod import;
pub mod module;
pub mod module_id;
pub mod semantic;
pub mod span;
pub mod statistics;
pub mod symbol;
mod class_enum_extraction;
mod code_quality_extraction;
pub mod package_json;
pub mod dependency_chain;

// ModuleGraph implementations
// Default: in-memory implementation (WASM-compatible)
mod memory;

// Storage-backed implementation (via fob-store when storage feature enabled)
#[cfg(feature = "storage")]
mod core;

// Re-export the appropriate ModuleGraph implementation
#[cfg(feature = "storage")]
pub use core::{
    ClassMemberInfo, EnumMemberInfo, ModuleGraph, NamespaceImportInfo, SideEffectImport,
    TypeOnlyImport, UnusedExport,
};
#[cfg(not(feature = "storage"))]
pub use memory::{
    ClassMemberInfo, EnumMemberInfo, ModuleGraph, NamespaceImportInfo, SideEffectImport,
    TypeOnlyImport,
};

// UnusedExport is defined in core.rs for storage feature,
// but we need to define it for memory feature too
#[cfg(not(feature = "storage"))]
#[derive(Debug, Clone)]
pub struct UnusedExport {
    pub module_id: module_id::ModuleId,
    pub export: export::Export,
}
pub use collection::{CollectionState, CollectedExport, CollectedImport, CollectedModule, CollectedImportSpecifier, parse_module_structure};
pub use export::{Export, ExportKind};
pub use external_dep::ExternalDependency;
pub use framework_rules::FrameworkRule;
pub use from_collection::CollectionGraphError;
pub use import::{Import, ImportKind, ImportSpecifier};
pub use module::{Module, SourceType};
pub use module_id::{ModuleId, ModuleIdError};
pub use span::SourceSpan;
pub use statistics::GraphStatistics;
pub use symbol::{
    ClassMemberMetadata, EnumMemberMetadata, EnumMemberValue, Symbol, SymbolKind, SymbolMetadata,
    SymbolSpan, SymbolStatistics, SymbolTable, UnreachableCode, UnusedSymbol, Visibility,
};
pub use package_json::{
    DependencyCoverage, DependencyType, PackageJson, TypeCoverage, UnusedDependency,
    extract_package_name,
};
pub use dependency_chain::{ChainAnalysis, DependencyChain};

#[cfg(test)]
mod tests;

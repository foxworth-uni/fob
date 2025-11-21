//! Core graph primitives shared across Joy's analyzers and builders.
//!
//! These types form the foundation for the ModuleGraph implementation described in
//! `design/02-graph-architecture.md`. They intentionally keep domain logic light so
//! higher-level phases can compose them without pulling in heavy dependencies.
//!
//! # #[tokio::main]
//! # async fn main() {
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
//! graph.add_module(utils).await;
//! graph.add_module(entry).await;
//! graph.add_dependency(entry_id, utils_id).await;
//!
//! assert!(graph.unused_exports().await.is_empty());
//! # }
//!
//! ```no_run
//! use fob::analyze;
//! use std::path::PathBuf;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let project_root = PathBuf::from("./examples/my-app");
//! // Note: analyze() uses current directory by default, or use Analyzer::new().cwd(...)
//! let result = analyze(&[project_root.join("src/index.js")]).await?;
//! println!("modules: {}", result.graph.len()?);
//! # Ok(())
//! # }
//! ```
//!

// Core graph types and primitives
mod class_enum_extraction;
mod code_quality_extraction;
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
    parse_module_structure, CollectedExport, CollectedImport, CollectedImportSpecifier,
    CollectedModule, CollectionState,
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
    extract_package_name, DependencyCoverage, DependencyType, PackageJson, TypeCoverage,
    UnusedDependency,
};
pub use span::SourceSpan;
pub use statistics::GraphStatistics;
pub use symbol::{
    ClassMemberMetadata, EnumMemberMetadata, EnumMemberValue, Symbol, SymbolKind, SymbolMetadata,
    SymbolSpan, SymbolStatistics, SymbolTable, UnreachableCode, UnusedSymbol, Visibility,
};

#[cfg(test)]
mod tests;

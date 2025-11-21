//! In-memory ModuleGraph implementation.
//!
//! This provides a HashMap-based graph storage that is fully in-memory,
//! making it compatible with WASM environments and simple to use.

mod chains;
mod construction;
mod exports;
mod framework;
mod graph;
mod imports;
mod mutations;
mod package_json;
mod queries;
mod serialization;
mod statistics;
mod symbols;
mod traversal;
mod types;

// Re-export types
pub use types::{
    ClassMemberInfo, EnumMemberInfo, NamespaceImportInfo, SideEffectImport, TypeOnlyImport,
};

// Re-export ModuleGraph
pub use graph::ModuleGraph;

// Import implementations - these add methods to ModuleGraph
// These wildcard imports are intentional - they add impl blocks to ModuleGraph
#[allow(unused_imports)]
use chains::*;
#[allow(unused_imports)]
use construction::*;
#[allow(unused_imports)]
use exports::*;
#[allow(unused_imports)]
use framework::*;
#[allow(unused_imports)]
use imports::*;
#[allow(unused_imports)]
use mutations::*;
#[allow(unused_imports)]
use package_json::*;
#[allow(unused_imports)]
use queries::*;
#[allow(unused_imports)]
use serialization::*;
#[allow(unused_imports)]
use statistics::*;
#[allow(unused_imports)]
use symbols::*;
#[allow(unused_imports)]
use traversal::*;

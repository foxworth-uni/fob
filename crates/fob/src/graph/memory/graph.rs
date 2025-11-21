//! Core ModuleGraph structure and inner state.

use std::sync::Arc;

use parking_lot::RwLock;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::super::external_dep::ExternalDependency;
use super::super::{Module, ModuleId};

/// In-memory module dependency graph.
///
/// This implementation uses HashMaps for fast lookups and is fully synchronous.
#[derive(Debug, Clone)]
pub struct ModuleGraph {
    pub(super) inner: Arc<RwLock<GraphInner>>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct GraphInner {
    /// All modules indexed by ID (wrapped in Arc for cheap cloning)
    pub modules: HashMap<ModuleId, Arc<Module>>,
    /// Forward edges: module -> its dependencies
    pub dependencies: HashMap<ModuleId, HashSet<ModuleId>>,
    /// Reverse edges: module -> modules that depend on it
    pub dependents: HashMap<ModuleId, HashSet<ModuleId>>,
    /// Entry point modules
    pub entry_points: HashSet<ModuleId>,
    /// External dependencies
    pub external_deps: HashMap<String, ExternalDependency>,
}


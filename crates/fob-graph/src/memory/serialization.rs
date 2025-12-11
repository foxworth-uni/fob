//! Serialization methods for ModuleGraph.

use super::super::external_dep::ExternalDependency;
use super::super::{Module, ModuleId};
use super::graph::{GraphInner, ModuleGraph};
use crate::{Error, Result};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::sync::Arc;

/// Helper to escape labels for DOT format.
fn escape_label(label: &str) -> String {
    label.replace('"', "\\\"")
}

impl ModuleGraph {
    /// Export the graph as DOT format for visualization.
    pub fn to_dot_format(&self) -> Result<String> {
        let mut output = String::from("digraph ModuleGraph {\n");
        let all_modules = self.modules()?;

        for module in &all_modules {
            output.push_str("    \"");
            output.push_str(&escape_label(&module.id.path_string()));
            output.push_str("\";\n");
        }

        for module in &all_modules {
            let deps = self.dependencies(&module.id)?;
            for target in deps {
                output.push_str("    \"");
                output.push_str(&escape_label(&module.id.path_string()));
                output.push_str("\" -> \"");
                output.push_str(&escape_label(&target.path_string()));
                output.push_str("\";\n");
            }
        }

        output.push_str("}\n");
        Ok(output)
    }

    /// Export the graph and modules to JSON.
    pub fn to_json(&self) -> Result<String> {
        let all_modules = self.modules()?;
        let entry_points = self.entry_points()?;
        let external_deps = self.external_dependencies()?;

        #[derive(serde::Serialize)]
        struct GraphJson {
            modules: Vec<Module>,
            entry_points: Vec<ModuleId>,
            external_deps: Vec<ExternalDependency>,
        }

        let graph_json = GraphJson {
            modules: all_modules,
            entry_points,
            external_deps,
        };

        serde_json::to_string_pretty(&graph_json)
            .map_err(|e| Error::InvalidConfig(format!("Failed to serialize graph: {e}")))
    }

    /// Serialize the graph to binary format using bincode.
    ///
    /// This includes a format version for forward compatibility and the full
    /// graph state (modules, dependencies, dependents, entry points, external deps).
    ///
    /// # Format Version
    ///
    /// The binary format starts with a u32 version number:
    /// - Version 1: Initial implementation with GraphInner serialization
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails or if the graph is poisoned.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        const FORMAT_VERSION: u32 = 1;

        let inner = self.inner.read();

        // Create a serializable representation of the graph
        #[derive(serde::Serialize)]
        struct SerializedGraph {
            version: u32,
            modules: HashMap<ModuleId, Arc<Module>>,
            dependencies: HashMap<ModuleId, HashSet<ModuleId>>,
            dependents: HashMap<ModuleId, HashSet<ModuleId>>,
            entry_points: HashSet<ModuleId>,
            external_deps: HashMap<String, ExternalDependency>,
        }

        let serialized = SerializedGraph {
            version: FORMAT_VERSION,
            modules: inner.modules.clone(),
            dependencies: inner.dependencies.clone(),
            dependents: inner.dependents.clone(),
            entry_points: inner.entry_points.clone(),
            external_deps: inner.external_deps.clone(),
        };

        bincode::serde::encode_to_vec(&serialized, bincode::config::standard())
            .map_err(|e| Error::InvalidConfig(format!("Failed to serialize graph to bytes: {e}")))
    }

    /// Deserialize the graph from binary format.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Deserialization fails
    /// - The format version is incompatible with the current implementation
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        const FORMAT_VERSION: u32 = 1;

        #[derive(serde::Deserialize)]
        struct SerializedGraph {
            version: u32,
            modules: HashMap<ModuleId, Arc<Module>>,
            dependencies: HashMap<ModuleId, HashSet<ModuleId>>,
            dependents: HashMap<ModuleId, HashSet<ModuleId>>,
            entry_points: HashSet<ModuleId>,
            external_deps: HashMap<String, ExternalDependency>,
        }

        let (serialized, _): (SerializedGraph, _) =
            bincode::serde::decode_from_slice(bytes, bincode::config::standard()).map_err(|e| {
                Error::InvalidConfig(format!("Failed to deserialize graph from bytes: {e}"))
            })?;

        // Validate format version
        if serialized.version != FORMAT_VERSION {
            return Err(Error::InvalidConfig(format!(
                "Incompatible graph format version: expected {}, got {}",
                FORMAT_VERSION, serialized.version
            )));
        }

        // Reconstruct the graph
        let inner = GraphInner {
            modules: serialized.modules,
            dependencies: serialized.dependencies,
            dependents: serialized.dependents,
            entry_points: serialized.entry_points,
            external_deps: serialized.external_deps,
        };

        Ok(ModuleGraph {
            inner: Arc::new(parking_lot::RwLock::new(inner)),
        })
    }
}

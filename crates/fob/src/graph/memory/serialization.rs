//! Serialization methods for ModuleGraph.

use super::graph::ModuleGraph;
use super::super::external_dep::ExternalDependency;
use super::super::{Module, ModuleId};
use crate::{Error, Result};

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
}


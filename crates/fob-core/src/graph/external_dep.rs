use serde::{Deserialize, Serialize};

use super::ModuleId;

/// Represents an external dependency (e.g. npm package) and modules that import it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalDependency {
    pub specifier: String,
    pub importers: Vec<ModuleId>,
}

impl ExternalDependency {
    pub fn new(specifier: impl Into<String>) -> Self {
        Self {
            specifier: specifier.into(),
            importers: Vec::new(),
        }
    }

    pub fn push_importer(&mut self, module_id: ModuleId) {
        self.importers.push(module_id);
    }

    pub fn extend_importers<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = ModuleId>,
    {
        self.importers.extend(iter);
    }
}

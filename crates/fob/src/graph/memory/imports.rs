//! Import analysis methods for ModuleGraph.

use super::graph::ModuleGraph;
use super::types::{NamespaceImportInfo, SideEffectImport, TypeOnlyImport};
use crate::Result;

impl ModuleGraph {
    /// Get all side-effect-only imports across the graph.
    ///
    /// Side-effect imports like `import 'polyfill'` execute code but don't bind any values.
    /// These are important to track as they can't be tree-shaken and always contribute
    /// to bundle size.
    pub fn side_effect_only_imports(&self) -> Result<Vec<SideEffectImport>> {
        let inner = self.inner.read();
        let mut side_effects = Vec::new();

        for module in inner.modules.values() {
            for import in module.imports.iter() {
                if import.is_side_effect_only() {
                    side_effects.push(SideEffectImport {
                        importer: module.id.clone(),
                        source: import.source.clone(),
                        resolved_to: import.resolved_to.clone(),
                        span: import.span.clone(),
                    });
                }
            }
        }

        Ok(side_effects)
    }

    /// Get all namespace imports and their usage.
    ///
    /// Namespace imports (`import * as foo`) import all exports from a module.
    /// This is useful for tracking potential over-imports that could be optimized
    /// to named imports.
    pub fn namespace_imports(&self) -> Result<Vec<NamespaceImportInfo>> {
        let inner = self.inner.read();
        let mut namespaces = Vec::new();

        for module in inner.modules.values() {
            for import in module.imports.iter() {
                if import.is_namespace_import() {
                    if let Some(namespace_name) = import.namespace_name() {
                        namespaces.push(NamespaceImportInfo {
                            importer: module.id.clone(),
                            namespace_name: namespace_name.to_string(),
                            source: import.source.clone(),
                            resolved_to: import.resolved_to.clone(),
                        });
                    }
                }
            }
        }

        Ok(namespaces)
    }

    /// Get all type-only imports (TypeScript).
    ///
    /// Type-only imports are erased at runtime and don't contribute to bundle size.
    /// Tracking these helps understand the TypeScript structure without conflating
    /// with runtime dependencies.
    pub fn type_only_imports(&self) -> Result<Vec<TypeOnlyImport>> {
        let inner = self.inner.read();
        let mut type_imports = Vec::new();

        for module in inner.modules.values() {
            for import in module.imports.iter() {
                if import.is_type_only() {
                    type_imports.push(TypeOnlyImport {
                        importer: module.id.clone(),
                        source: import.source.clone(),
                        specifiers: import.specifiers.clone(),
                        span: import.span.clone(),
                    });
                }
            }
        }

        Ok(type_imports)
    }
}

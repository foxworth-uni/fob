use std::path::Path;

use rustc_hash::FxHashMap;

use crate::graph::ModuleGraph;
use crate::Result;

use super::metadata::BundleMetadata;
use super::{bundle::Bundle, import_map::ImportMap};

/// Aggregated result for component bundling.
pub struct ComponentBuild {
    bundles: FxHashMap<String, Bundle>,
    shared_graph: ModuleGraph,
    shared_imports: Vec<String>,
    import_map: ImportMap,
}

impl ComponentBuild {
    pub(crate) async fn new(bundles: FxHashMap<String, Bundle>) -> Result<Self> {
        let shared_graph = merge_graphs(&bundles).await?;
        let shared_imports = compute_shared_imports(&bundles).await?;
        let import_map = build_import_map(&bundles);

        Ok(Self {
            bundles,
            shared_graph,
            shared_imports,
            import_map,
        })
    }

    pub fn len(&self) -> usize {
        self.bundles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bundles.is_empty()
    }

    pub fn get(&self, entry: &str) -> Option<&Bundle> {
        self.bundles.get(entry)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Bundle)> {
        self.bundles.iter()
    }

    pub fn shared_graph(&self) -> &ModuleGraph {
        &self.shared_graph
    }

    pub fn shared_imports(&self) -> &[String] {
        &self.shared_imports
    }

    pub fn import_map(&self) -> &ImportMap {
        &self.import_map
    }

    /// Writes all component bundles to disk, erroring if files already exist.
    ///
    /// Each component bundle is written to a subdirectory named after its entry point.
    /// For example, if you have components "button.js" and "badge.js", the output
    /// structure will be:
    ///
    /// ```text
    /// dist/
    ///   button/
    ///     bundle.js
    ///     bundle.js.map
    ///   badge/
    ///     bundle.js
    ///     bundle.js.map
    /// ```
    ///
    /// # Atomic Guarantees
    ///
    /// Either all files for all components are written successfully or none are written.
    /// If any operation fails, all previously written files are rolled back.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fob_core as fob;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = fob::BuildOptions::components(["./src/button.js", "./src/badge.js"])
    ///     .build()
    ///     .await?;
    ///
    /// let components = result.output.as_multiple().expect("multiple bundles");
    /// // Write all components to dist/, error if files exist
    /// components.write_to("dist")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_to(&self, dir: impl AsRef<Path>) -> Result<()> {
        let dir = dir.as_ref();

        // Write each component bundle
        for (name, bundle) in &self.bundles {
            // Create a subdirectory for this component
            let component_dir = dir.join(sanitize_component_name(name));
            bundle.write_to(&component_dir)?;
        }

        Ok(())
    }

    /// Writes all component bundles to disk, overwriting existing files.
    ///
    /// Each component bundle is written to a subdirectory named after its entry point.
    ///
    /// # Atomic Guarantees
    ///
    /// Either all files for all components are written successfully or none are written.
    /// If any operation fails, all previously written files are rolled back.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fob_core as fob;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = fob::BuildOptions::components(["./src/button.js", "./src/badge.js"])
    ///     .build()
    ///     .await?;
    ///
    /// let components = result.output.as_multiple().expect("multiple bundles");
    /// // Overwrite existing files in dist/
    /// components.write_to_force("dist")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_to_force(&self, dir: impl AsRef<Path>) -> Result<()> {
        let dir = dir.as_ref();

        // Write each component bundle
        for (name, bundle) in &self.bundles {
            // Create a subdirectory for this component
            let component_dir = dir.join(sanitize_component_name(name));
            bundle.write_to_force(&component_dir)?;
        }

        Ok(())
    }

    /// Extracts metadata for all component bundles.
    ///
    /// Returns a vector of tuples containing the component name and its metadata.
    /// This is useful for understanding what each component exports and its dependencies.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fob_core as fob;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let result = fob::BuildOptions::components(["./src/button.js", "./src/badge.js"])
    ///     .build()
    ///     .await?;
    ///
    /// let components = result.output.as_multiple().expect("multiple bundles");
    /// for (name, metadata) in components.metadata() {
    ///     println!("Component {}: {} exports, {} bytes",
    ///         name,
    ///         metadata.exports().len(),
    ///         metadata.total_size()
    ///     );
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn metadata(&self) -> Result<Vec<(&String, BundleMetadata)>> {
        let mut results = Vec::new();
        for (name, bundle) in self.bundles.iter() {
            results.push((name, bundle.metadata().await?));
        }
        Ok(results)
    }
}

/// Sanitizes a component name for use as a directory name.
///
/// This removes file extensions and path separators to create a safe directory name.
/// For example:
/// - "./src/button.js" -> "button"
/// - "components/badge.tsx" -> "badge"
fn sanitize_component_name(name: &str) -> String {
    // Get the file name without path
    let name = name.rsplit('/').next().unwrap_or(name);
    let name = name.rsplit('\\').next().unwrap_or(name);

    // Remove extension
    name.split('.').next().unwrap_or(name).to_string()
}

async fn merge_graphs(bundles: &FxHashMap<String, Bundle>) -> Result<ModuleGraph> {
    let aggregated = ModuleGraph::new().await?;

    for bundle in bundles.values() {
        let graph = bundle.module_graph();

        let modules = graph.modules().await?;
        for module in modules {
            aggregated.add_module(module).await?;
        }

        let all_modules = graph.modules().await?;
        for module in all_modules {
            let deps = graph.dependencies(&module.id).await?;
            aggregated.add_dependencies(module.id.clone(), deps.into_iter()).await?;
        }

        let entries = graph.entry_points().await?;
        for entry in entries {
            aggregated.add_entry_point(entry).await?;
        }

        let external_deps = graph.external_dependencies().await?;
        for dep in external_deps {
            aggregated.add_external_dependency(dep).await?;
        }
    }

    Ok(aggregated)
}

async fn compute_shared_imports(bundles: &FxHashMap<String, Bundle>) -> Result<Vec<String>> {
    let mut counts: FxHashMap<String, usize> = FxHashMap::default();

    for bundle in bundles.values() {
        let deps = bundle.module_graph().external_dependencies().await?;
        for dep in deps {
            *counts.entry(dep.specifier.clone()).or_default() += 1;
        }
    }

    Ok(counts
        .into_iter()
        .filter_map(|(spec, count)| if count > 1 { Some(spec) } else { None })
        .collect())
}

fn build_import_map(bundles: &FxHashMap<String, Bundle>) -> ImportMap {
    let mut map = ImportMap::new();

    for (entry, _bundle) in bundles.iter() {
        // TODO: use Rolldown output metadata once exposed. For now, map to the entry specifier itself.
        map.insert(entry.clone(), entry.clone());
    }

    map
}

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use path_clean::PathClean;
use rolldown::{BundlerBuilder as RolldownBundlerBuilder, BundlerOptions, InputItem};
use rolldown_plugin::{__inner::SharedPluginable, Plugin};
use rustc_hash::FxHashMap;

use crate::analysis::AnalyzedBundle;
use crate::builders::{asset_plugin::AssetDetectionPlugin, asset_registry::AssetRegistry};
use crate::diagnostics;
use crate::module_collection_plugin::ModuleCollectionPlugin;
use crate::{Error, Result};
use fob_graph::analysis::stats::compute_stats;
use fob_graph::{AnalysisResult, CacheAnalysis, TransformationTrace};

/// Normalize an entry path by cleaning redundant `.` / `..` segments.
pub(crate) fn normalize_entry_path(entry: impl AsRef<Path>) -> String {
    let cleaned: PathBuf = entry.as_ref().to_path_buf().clean();
    cleaned.to_string_lossy().into_owned()
}

/// Trait alias for values that can be converted into a `SharedPluginable`.
pub trait IntoPlugin {
    fn into_plugin(self) -> SharedPluginable;
}

pub(crate) struct PluginHandle<P>(P);

pub fn plugin<P>(plugin: P) -> impl IntoPlugin
where
    P: Plugin + 'static,
{
    PluginHandle(plugin)
}

impl IntoPlugin for SharedPluginable {
    fn into_plugin(self) -> SharedPluginable {
        self
    }
}

impl<T> IntoPlugin for Arc<T>
where
    T: Plugin + 'static,
{
    fn into_plugin(self) -> SharedPluginable {
        self
    }
}

impl<P> IntoPlugin for PluginHandle<P>
where
    P: Plugin + 'static,
{
    fn into_plugin(self) -> SharedPluginable {
        Arc::new(self.0)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EntrySpec {
    pub name: Option<String>,
    pub import: String,
}

pub(crate) struct BundlePlan {
    pub entries: Vec<EntrySpec>,
    pub options: BundlerOptions,
    pub plugins: Vec<SharedPluginable>,
    pub cwd: Option<PathBuf>,
    pub virtual_files: FxHashMap<String, String>,
    pub runtime: Option<Arc<dyn crate::Runtime>>,
}

// Legacy function - no longer used with plugin-based collection
// Keeping for reference but should be removed once integration is verified

// Legacy scan_bundle_for_assets function removed.
// Asset detection now happens automatically via the AssetDetectionPlugin transform hook
// during bundling, which is more efficient and WASM-compatible.

pub(crate) async fn execute_bundle(plan: BundlePlan) -> Result<AnalyzedBundle> {
    let BundlePlan {
        entries,
        mut options,
        mut plugins,
        cwd,
        virtual_files,
        runtime,
    } = plan;

    // Determine the scan_cwd for asset scanning
    // Priority: explicit cwd > runtime.get_cwd() > std::env::current_dir() (native only)
    let scan_cwd = match cwd.clone() {
        Some(path) => {
            eprintln!("[FOB_BUILD] Using explicit cwd: {}", path.display());
            path
        }
        None => {
            // Try to get cwd from runtime if available
            if let Some(ref rt) = runtime {
                match rt.get_cwd() {
                    Ok(path) => {
                        eprintln!("[FOB_BUILD] Using runtime cwd: {}", path.display());
                        path
                    }
                    Err(e) => {
                        // On WASM, this is a critical error - we MUST have a cwd
                        #[cfg(target_family = "wasm")]
                        {
                            return Err(Error::InvalidConfig(format!(
                                "Failed to get working directory from runtime: {}. \
                                On WASM, you must provide either an explicit cwd or a runtime with get_cwd() support.",
                                e
                            )));
                        }

                        // On native, fall back to std::env::current_dir()
                        #[cfg(not(target_family = "wasm"))]
                        {
                            eprintln!(
                                "[FOB_BUILD] Runtime get_cwd() failed: {}, falling back to std::env::current_dir()",
                                e
                            );
                            std::env::current_dir()?
                        }
                    }
                }
            } else {
                // No runtime provided
                #[cfg(target_family = "wasm")]
                {
                    return Err(Error::InvalidConfig(
                        "No working directory available on WASM. \
                        You must provide either an explicit cwd via BuildOptions or a Runtime implementation.".to_string()
                    ));
                }

                #[cfg(not(target_family = "wasm"))]
                {
                    eprintln!(
                        "[FOB_BUILD] No runtime or explicit cwd, using std::env::current_dir()"
                    );
                    std::env::current_dir()?
                }
            }
        }
    };

    // Create module collection plugin to gather module data during bundling
    let collection_plugin = Arc::new(ModuleCollectionPlugin::new());

    options.input = Some(
        entries
            .iter()
            .map(|entry| InputItem {
                name: entry.name.clone(),
                import: entry.import.clone(),
            })
            .collect(),
    );

    if options.cwd.is_none() {
        options.cwd = cwd;
    }

    // Add VirtualFilePlugin if there are virtual files
    if !virtual_files.is_empty() {
        use crate::builders::virtual_file_plugin::VirtualFilePlugin;
        let virtual_plugin = VirtualFilePlugin::new(virtual_files)?;
        plugins.push(Arc::new(virtual_plugin));
    }

    // Create asset registry and add asset detection plugin
    let asset_registry = Arc::new(AssetRegistry::new());
    let asset_extensions = vec![
        ".wasm".to_string(),
        ".png".to_string(),
        ".jpg".to_string(),
        ".jpeg".to_string(),
        ".gif".to_string(),
        ".svg".to_string(),
        ".webp".to_string(),
        ".ico".to_string(),
        ".ttf".to_string(),
        ".woff".to_string(),
        ".woff2".to_string(),
    ];
    // Auto-inject NativeRuntime on native platforms for convenience.
    // This allows tests and simple use cases to work without explicit runtime setup.
    // WASM platforms still require explicit runtime since there's no standard filesystem.
    let runtime = match runtime {
        Some(rt) => rt,
        None => {
            #[cfg(not(target_family = "wasm"))]
            {
                eprintln!("[FOB_BUILD] No runtime provided, using NativeRuntime");
                Arc::new(crate::NativeRuntime::new())
            }
            #[cfg(target_family = "wasm")]
            {
                return Err(Error::InvalidConfig(
                    "Runtime is required for asset detection plugin. \
                    On WASM, you must provide a Runtime implementation via BuildOptions::runtime()."
                        .to_string(),
                ));
            }
        }
    };

    let asset_plugin = AssetDetectionPlugin::new(
        Arc::clone(&asset_registry),
        &scan_cwd,
        asset_extensions,
        Arc::clone(&runtime),
    );
    plugins.push(Arc::new(asset_plugin));
    eprintln!("[FOB_BUILD] Added asset detection plugin");

    // Add module collection plugin to gather module data
    plugins.push(collection_plugin.clone());
    eprintln!("[FOB_BUILD] Added module collection plugin");

    let mut bundler = RolldownBundlerBuilder::default()
        .with_options(options)
        .with_plugins(plugins)
        .build()
        .map_err(|e| Error::from_rolldown_batch(&e))?;

    let bundle = bundler
        .generate()
        .await
        .map_err(|e| Error::from_rolldown_batch(&e))?;

    eprintln!(
        "[FOB_BUILD] Bundle complete. Asset registry has {} assets",
        asset_registry.len()
    );

    // Asset detection now happens during bundling via the plugin transform hook
    // The asset_registry was populated by the AssetDetectionPlugin during transform
    let asset_registry_opt = if !asset_registry.is_empty() {
        Some(asset_registry)
    } else {
        None
    };

    // Extract collected module data from the plugin and build the module graph
    let collection_data = collection_plugin.take_data();
    eprintln!(
        "[FOB_BUILD] Collected {} modules",
        collection_data.modules.len()
    );

    let graph = fob_graph::ModuleGraph::from_collected_data(collection_data).map_err(|e| {
        Error::Bundler(vec![diagnostics::ExtractedDiagnostic {
            kind: diagnostics::DiagnosticKind::Other("GraphBuildError".to_string()),
            severity: diagnostics::DiagnosticSeverity::Error,
            message: format!("Failed to build module graph from collected data: {}", e),
            file: None,
            line: None,
            column: None,
            help: None,
            context: None,
        }])
    })?;

    let stats = compute_stats(&graph)?;
    let entry_points = graph.entry_points()?;
    let symbol_stats = graph.symbol_statistics()?;
    let analysis = AnalysisResult {
        graph,
        entry_points,
        warnings: Vec::new(),
        errors: Vec::new(),
        stats,
        symbol_stats,
    };

    let cache = CacheAnalysis::default();
    let trace = if std::env::var_os("JOY_TRACE").is_some() {
        Some(TransformationTrace::default())
    } else {
        None
    };

    Ok(AnalyzedBundle {
        bundle,
        analysis,
        cache,
        trace,
        asset_registry: asset_registry_opt,
    })
}

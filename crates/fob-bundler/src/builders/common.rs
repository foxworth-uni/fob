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
use crate::plugins::{PluginPhase, PluginRegistry};
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

pub(crate) async fn execute_bundle(plan: BundlePlan) -> Result<AnalyzedBundle> {
    let BundlePlan {
        entries,
        mut options,
        plugins,
        cwd,
        virtual_files,
        runtime,
    } = plan;

    // Determine the scan_cwd for asset scanning
    // Priority: explicit cwd > runtime.get_cwd() > std::env::current_dir() (native only)
    let scan_cwd = match cwd.clone() {
        Some(path) => path,
        None => {
            // Try to get cwd from runtime if available
            if let Some(ref rt) = runtime {
                match rt.get_cwd() {
                    Ok(path) => path,
                    Err(_e) => {
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
        options.cwd = cwd.clone();
    }

    // Create BundlerRuntime - wraps virtual files + filesystem
    // Auto-inject NativeRuntime on native platforms for convenience.
    // This allows tests and simple use cases to work without explicit runtime setup.
    // WASM platforms still require explicit runtime since there's no standard filesystem.
    // Note: BundlerRuntime handles filesystem access internally, so we don't need to store
    // the base runtime - it's only used here to ensure one exists for WASM platforms.
    match runtime {
        Some(_rt) => {
            // Runtime provided - BundlerRuntime will use filesystem directly
        }
        None => {
            #[cfg(not(target_family = "wasm"))]
            {
                // On native, BundlerRuntime can use filesystem directly
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
    }

    // Create BundlerRuntime with cwd
    let bundler_runtime = Arc::new(crate::runtime::BundlerRuntime::new(scan_cwd.clone()));

    // Register virtual files in BundlerRuntime (with validation)
    for (path, content) in &virtual_files {
        // Validate virtual file path
        if path.contains('\0') {
            return Err(Error::InvalidOutputPath(
                "Virtual file path contains null byte".to_string(),
            ));
        }
        if path.len() > 4096 {
            return Err(Error::InvalidOutputPath(format!(
                "Virtual file path too long: {} bytes (max 4096)",
                path.len()
            )));
        }

        // Validate virtual file content size (1MB limit)
        const MAX_VIRTUAL_FILE_SIZE: usize = 1024 * 1024;
        if content.len() > MAX_VIRTUAL_FILE_SIZE {
            return Err(Error::WriteFailure(format!(
                "Virtual file content too large: {} bytes (max {} bytes)",
                content.len(),
                MAX_VIRTUAL_FILE_SIZE
            )));
        }

        bundler_runtime.add_virtual_file(path, content.as_bytes());
    }

    // Create RuntimeFilePlugin (Virtual phase - serves virtual files)
    let runtime_file_plugin =
        crate::builders::runtime_file_plugin::RuntimeFilePlugin::new(Arc::clone(&bundler_runtime));

    // Create asset registry and asset detection plugin (Assets phase)
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

    // Use BundlerRuntime for asset detection (it handles virtual files + filesystem)
    let runtime: Arc<dyn crate::Runtime> = bundler_runtime;

    let asset_plugin = AssetDetectionPlugin::new(
        Arc::clone(&asset_registry),
        &scan_cwd,
        asset_extensions,
        Arc::clone(&runtime),
    );

    // Build plugin registry with guaranteed ordering by phase:
    // Virtual (0) → Transform (20) → Assets (30) → PostProcess (100)
    let mut registry = PluginRegistry::new();

    // Built-in plugins use their FobPlugin::phase() for ordering
    registry.add(runtime_file_plugin); // Virtual = 0
    registry.add(asset_plugin); // Assets = 30
    // collection_plugin is already Arc<T>, so use add_with_phase
    registry.add_with_phase(collection_plugin.clone(), PluginPhase::PostProcess);

    // User plugins default to Transform phase
    for plugin in plugins {
        registry.add_with_phase(plugin, PluginPhase::Transform);
    }

    // Convert to ordered Vec for Rolldown (sorted by phase)
    let ordered_plugins = registry.into_rolldown_plugins();

    let mut bundler = RolldownBundlerBuilder::default()
        .with_options(options)
        .with_plugins(ordered_plugins)
        .build()
        .map_err(|e| Error::from_rolldown_batch(&e))?;

    let bundle = bundler
        .generate()
        .await
        .map_err(|e| Error::from_rolldown_batch(&e))?;

    let asset_registry_opt = if !asset_registry.is_empty() {
        Some(asset_registry)
    } else {
        None
    };

    // Extract collected module data from the plugin and build the module graph
    let collection_data = collection_plugin.take_data();

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
            error_chain: Vec::new(),
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

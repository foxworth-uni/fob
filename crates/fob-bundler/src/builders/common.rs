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
    pub cache: Option<crate::cache::CacheConfig>,
    pub incremental: Option<crate::builders::unified::primitives::IncrementalConfig>,
}

pub(crate) async fn execute_bundle(plan: BundlePlan) -> Result<AnalyzedBundle> {
    // Clone data needed for cache key computation before consuming the plan
    let cache_plan_for_key = if plan.cache.is_some() {
        Some(BundlePlan {
            entries: plan.entries.clone(),
            options: plan.options.clone(),
            plugins: plan.plugins.clone(),
            cwd: plan.cwd.clone(),
            virtual_files: plan.virtual_files.clone(),
            runtime: plan.runtime.clone(),
            cache: None,       // Don't include cache config in the key
            incremental: None, // Don't include incremental config in the key
        })
    } else {
        None
    };

    let BundlePlan {
        entries,
        mut options,
        plugins,
        cwd,
        virtual_files,
        runtime,
        cache: cache_config,
        incremental: incremental_config,
    } = plan;

    // Try to load from cache if enabled
    if let Some(ref config) = cache_config {
        if !config.should_force_rebuild() {
            if let Some(ref key_plan) = cache_plan_for_key {
                match try_load_from_cache(key_plan, config) {
                    Ok(cached) => {
                        // Cache hit - return the cached result
                        return Ok(cached);
                    }
                    Err(e) => {
                        // Cache miss or error - continue with build
                        // Errors are non-fatal, just log them
                        eprintln!("Cache miss or error: {}", e);
                    }
                }
            }
        }
    }

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

    // Try to use incremental cache if enabled
    let graph = if let Some(ref inc_config) = incremental_config {
        // Try to load cached graph
        match try_load_incremental_graph(inc_config, &entries) {
            Ok(Some(cached_graph)) => {
                // Cache hit - reuse the graph
                eprintln!("Incremental cache hit - reusing cached module graph");
                cached_graph
            }
            Ok(None) => {
                // Cache miss or first build - construct graph normally
                eprintln!("Incremental cache miss - building module graph from scratch");
                let graph =
                    fob_graph::ModuleGraph::from_collected_data(collection_data).map_err(|e| {
                        Error::Bundler(vec![diagnostics::ExtractedDiagnostic {
                            kind: diagnostics::DiagnosticKind::Other("GraphBuildError".to_string()),
                            severity: diagnostics::DiagnosticSeverity::Error,
                            message: format!(
                                "Failed to build module graph from collected data: {}",
                                e
                            ),
                            file: None,
                            line: None,
                            column: None,
                            help: None,
                            context: None,
                            error_chain: Vec::new(),
                        }])
                    })?;

                // Save to incremental cache
                if let Err(e) = try_save_incremental_graph(inc_config, &graph) {
                    eprintln!("Warning: Failed to save incremental cache: {}", e);
                }

                graph
            }
            Err(e) => {
                // Cache error (non-fatal) - fall back to normal build
                eprintln!("Incremental cache error (non-fatal): {}", e);
                fob_graph::ModuleGraph::from_collected_data(collection_data).map_err(|e| {
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
                })?
            }
        }
    } else {
        // No incremental caching - build graph normally
        fob_graph::ModuleGraph::from_collected_data(collection_data).map_err(|e| {
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
        })?
    };

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

    let result = AnalyzedBundle {
        bundle,
        analysis,
        cache,
        trace,
        asset_registry: asset_registry_opt,
    };

    // Save to cache if enabled
    if let Some(ref config) = cache_config {
        if let Some(ref key_plan) = cache_plan_for_key {
            if let Err(e) = try_save_to_cache(key_plan, config, &result) {
                // Cache save errors are non-fatal, just log them
                eprintln!("Failed to save to cache: {}", e);
            }
        }
    }

    Ok(result)
}

/// Try to load a build result from cache.
fn try_load_from_cache(
    plan: &BundlePlan,
    config: &crate::cache::CacheConfig,
) -> Result<AnalyzedBundle> {
    use crate::BundleOutput;
    use crate::cache::{compute_cache_key, open_store, try_load};

    // Open cache store
    let store = open_store(&config.dir)
        .map_err(|e| Error::InvalidConfig(format!("Failed to open cache store: {}", e)))?;

    // Compute cache key
    let key = compute_cache_key(plan, config)
        .map_err(|e| Error::InvalidConfig(format!("Failed to compute cache key: {}", e)))?;

    // Try to load from cache
    let cached =
        try_load(&store, &key).map_err(|e| Error::InvalidConfig(format!("Cache error: {}", e)))?;

    // Validate metadata
    if !cached.metadata.is_compatible() {
        return Err(Error::InvalidConfig(format!(
            "Cache version mismatch: expected {}, found {}",
            crate::cache::serialize::CACHE_FORMAT_VERSION,
            cached.metadata.format_version
        )));
    }

    // Convert cached build back to AnalyzedBundle
    let bundle_output = BundleOutput {
        assets: cached
            .outputs
            .into_iter()
            .map(|o| o.into_rolldown())
            .collect(),
        warnings: vec![],
    };

    // Create a minimal graph for cached results
    // The graph JSON is preserved but not deserialized (ModuleGraph doesn't implement Deserialize)
    let graph = fob_graph::ModuleGraph::new()
        .map_err(|e| Error::InvalidConfig(format!("Failed to create module graph: {}", e)))?;

    // Convert entry_points from String to ModuleId
    let entry_points: Vec<fob_graph::ModuleId> = cached
        .entry_points
        .into_iter()
        .map(|id| {
            fob_graph::ModuleId::new(id)
                .unwrap_or_else(|_| fob_graph::ModuleId::new("unknown").unwrap())
        })
        .collect();

    let analysis = fob_graph::AnalysisResult {
        graph,
        entry_points,
        warnings: cached.warnings,
        errors: cached.errors,
        stats: fob_graph::GraphStatistics::default(),
        symbol_stats: fob_graph::SymbolStatistics::new(0, 0),
    };

    // Reconstruct asset registry if present
    let asset_registry = if !cached.assets.is_empty() {
        let registry = crate::builders::asset_registry::AssetRegistry::new();
        for asset_info in cached.assets {
            registry.register(
                PathBuf::from(asset_info.source_path),
                asset_info.referrer,
                asset_info.specifier,
            );
        }
        Some(Arc::new(registry))
    } else {
        None
    };

    Ok(AnalyzedBundle {
        bundle: bundle_output,
        analysis,
        cache: cached.cache,
        trace: cached.trace,
        asset_registry,
    })
}

/// Try to save a build result to cache.
fn try_save_to_cache(
    plan: &BundlePlan,
    config: &crate::cache::CacheConfig,
    result: &AnalyzedBundle,
) -> Result<()> {
    use crate::cache::{
        compute_cache_key, open_store,
        serialize::{BuildComponents, CachedBuild},
        try_save,
    };

    // Open cache store
    let store = open_store(&config.dir)
        .map_err(|e| Error::InvalidConfig(format!("Failed to open cache store: {}", e)))?;

    // Compute cache key
    let key = compute_cache_key(plan, config)
        .map_err(|e| Error::InvalidConfig(format!("Failed to compute cache key: {}", e)))?;

    // For now, use an empty JSON object for the graph since ModuleGraph doesn't implement Serialize
    // This is acceptable because the cache is primarily for bundle output, not graph analysis
    let graph_json = "{}".to_string();

    // Convert entry_points to strings
    let entry_points_str: Vec<String> = result
        .analysis
        .entry_points
        .iter()
        .map(|id| id.to_string())
        .collect();

    // Create cached build
    let cached = CachedBuild::from_components(BuildComponents {
        bundle: &result.bundle,
        graph_json,
        entry_points: entry_points_str,
        warnings: result.analysis.warnings.clone(),
        errors: result.analysis.errors.clone(),
        cache: &result.cache,
        trace: result.trace.as_ref(),
        asset_registry: result.asset_registry.as_ref(),
    });

    // Save to cache
    try_save(&store, &key, &cached)
        .map_err(|e| Error::InvalidConfig(format!("Failed to save to cache: {}", e)))?;

    Ok(())
}

/// Try to load cached module graph using incremental caching.
///
/// Returns:
/// - Ok(Some(graph)) if cache hit and valid
/// - Ok(None) if cache miss or invalid
/// - Err if fatal error (though incremental errors should be non-fatal)
fn try_load_incremental_graph(
    config: &crate::builders::unified::primitives::IncrementalConfig,
    entries: &[EntrySpec],
) -> Result<Option<fob_graph::ModuleGraph>> {
    use crate::cache::changes::ChangeDetector;
    use crate::cache::incremental::IncrementalCache;

    // Load incremental cache
    let cache = match IncrementalCache::load(&config.cache_dir) {
        Ok(Some(cache)) => cache,
        Ok(None) => {
            // No cache file exists (first build)
            return Ok(None);
        }
        Err(e) => {
            eprintln!("Warning: Failed to load incremental cache: {}", e);
            return Ok(None);
        }
    };

    // Convert entry specs to paths
    let entry_paths: Vec<PathBuf> = entries
        .iter()
        .map(|spec| PathBuf::from(&spec.import))
        .collect();

    // Check if cache is valid for current entries
    if !cache.is_valid_for(&entry_paths) {
        eprintln!("Incremental cache invalid: entry points or Rolldown version changed");
        return Ok(None);
    }

    // Get the cached graph
    let Some(ref graph) = cache.graph else {
        return Ok(None);
    };

    // Create change detector from cached hashes
    let detector = ChangeDetector::from_graph(graph)?;

    // Compute current file hashes
    let modules = graph.modules()?;
    let mut current_hashes = Vec::new();
    for module in modules {
        if module.id.is_virtual() {
            continue;
        }

        match std::fs::read(&module.path) {
            Ok(contents) => {
                let hash = blake3::hash(&contents);
                current_hashes.push((module.id.clone(), *hash.as_bytes()));
            }
            Err(_) => {
                // File read error - treat as cache invalidation
                eprintln!(
                    "Warning: Failed to read file {:?} for change detection. Invalidating cache.",
                    module.path
                );
                return Ok(None);
            }
        }
    }

    // Detect changes
    let changes = detector.detect_changes(&current_hashes);

    // If any changes detected, invalidate cache
    if changes.has_changes() {
        eprintln!(
            "Incremental cache invalid: {} files changed",
            changes.modified.len() + changes.added.len() + changes.removed.len()
        );
        return Ok(None);
    }

    // Cache is valid - return the graph
    Ok(Some(graph.clone()))
}

/// Save module graph to incremental cache.
fn try_save_incremental_graph(
    config: &crate::builders::unified::primitives::IncrementalConfig,
    graph: &fob_graph::ModuleGraph,
) -> Result<()> {
    use crate::cache::changes::ChangeDetector;
    use crate::cache::incremental::IncrementalCache;

    // Create change detector from current graph
    let detector = ChangeDetector::from_graph(graph)?;

    // Create incremental cache
    let cache = IncrementalCache {
        graph: Some(graph.clone()),
        module_hashes: detector.module_hashes.clone(),
        rolldown_version: crate::cache::incremental::IncrementalCache::new().rolldown_version,
        format_version: crate::cache::incremental::IncrementalCache::FORMAT_VERSION,
    };

    // Save to disk
    cache
        .save(&config.cache_dir)
        .map_err(|e| Error::InvalidConfig(format!("Failed to save incremental cache: {}", e)))?;

    Ok(())
}

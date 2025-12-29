//! Build execution and dispatch logic.
//!
//! This module contains the internal implementation that dispatches
//! build operations to the appropriate execution path based on the
//! BuildOptions configuration.

use rolldown::{BundlerOptions, GlobalsOutputOption, IsExternal, Platform, ResolveOptions};
use rustc_hash::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::Result;
use crate::analysis::AnalyzedBundle;
use crate::builders::common::{BundlePlan, EntrySpec, execute_bundle};
use crate::builders::unified::primitives::{EntryMode, ExternalConfig};
use crate::builders::unified::{BuildOptions, BuildOutput, BuildResult, EntryPoints, MinifyLevel};
use crate::target::ExportConditions;

#[cfg(feature = "dts-generation")]
use crate::plugins::DtsEmitPlugin;

/// Execute a build with the given options.
///
/// Dispatches to the appropriate execution path based on EntryMode:
/// - `Shared`: All entries share one bundle context (with optional code splitting)
/// - `Isolated`: Each entry is built independently
pub async fn execute_build(options: BuildOptions) -> Result<BuildResult> {
    match options.entry_mode {
        EntryMode::Shared => execute_unified_build(options).await,
        EntryMode::Isolated => execute_separate_builds(options).await,
    }
}

/// Execute a unified build (all entries share one bundle context).
///
/// This mode supports code splitting via the chunking strategy.
async fn execute_unified_build(options: BuildOptions) -> Result<BuildResult> {
    let entries = match &options.entry {
        EntryPoints::Single(e) => vec![EntrySpec {
            name: None,
            import: e.clone(),
        }],
        EntryPoints::Multiple(v) => v
            .iter()
            .map(|e| EntrySpec {
                name: None,
                import: e.clone(),
            })
            .collect(),
        EntryPoints::Named(m) => m
            .iter()
            .map(|(name, import)| EntrySpec {
                name: Some(name.clone()),
                import: import.clone(),
            })
            .collect(),
    };

    let mut rolldown_options = configure_rolldown_options(&options);

    // Apply code splitting configuration
    rolldown_options.advanced_chunks = options
        .code_splitting
        .as_ref()
        .map(|c| c.to_rolldown_options());

    // Add DTS plugin if enabled
    #[cfg(feature = "dts-generation")]
    let plugins = if let Some(dts_opts) = &options.dts {
        let mut plugins = Vec::new();
        if let Some(plugin) = configure_dts_plugin(dts_opts, &entries) {
            plugins.push(plugin);
        }
        plugins
    } else {
        Vec::new()
    };

    #[cfg(not(feature = "dts-generation"))]
    let plugins = Vec::new();

    let plan = BundlePlan {
        entries,
        options: rolldown_options,
        plugins,
        cwd: options.cwd.clone(),
        virtual_files: options.virtual_files.clone(),
        runtime: options.runtime.clone(),
        cache: options.cache.clone(),
        incremental: options.incremental.clone(),
    };

    let analyzed = execute_bundle(plan).await?;
    Ok(build_result_from_analyzed(analyzed, BuildOutput::Single))
}

/// Execute separate builds (each entry is built independently).
///
/// No code sharing between bundles; each is self-contained.
/// On native platforms, builds run in parallel for 2-3x speedup.
/// On WASM, builds run sequentially (single-threaded).
async fn execute_separate_builds(options: BuildOptions) -> Result<BuildResult> {
    let entries = match &options.entry {
        EntryPoints::Multiple(v) => v.clone(),
        EntryPoints::Named(m) => m.values().cloned().collect::<Vec<_>>(),
        EntryPoints::Single(_) => {
            // validate() prevents this
            unreachable!("EntryMode::Isolated with Single entry caught by validate()")
        }
    };

    // Execute builds (parallel on native, sequential on WASM)
    let results = execute_builds_concurrent(&options, &entries).await;

    // Merge results in original order for determinism
    merge_build_results(results, &entries)
}

/// Execute builds concurrently using tokio task spawning (native only).
///
/// Uses `JoinSet` with `Semaphore` for structured concurrency with bounded parallelism.
#[cfg(not(target_family = "wasm"))]
async fn execute_builds_concurrent(
    options: &BuildOptions,
    entries: &[String],
) -> Vec<(String, Result<AnalyzedBundle>)> {
    use tokio::sync::Semaphore;
    use tokio::task::JoinSet;

    let max_parallel = options
        .max_parallel_builds
        .unwrap_or_else(|| num_cpus::get().min(8));

    let mut join_set = JoinSet::new();
    let semaphore = Arc::new(Semaphore::new(max_parallel));

    // Spawn tasks with semaphore to limit concurrency
    for entry in entries.iter() {
        let entry = entry.clone();
        let opts = options.clone();
        let permit = Arc::clone(&semaphore);

        join_set.spawn(async move {
            // Acquire permit before starting build
            let _permit = permit
                .acquire()
                .await
                .expect("semaphore closed unexpectedly");
            let result = build_single_component(&opts, &entry).await;
            (entry, result)
        });
    }

    // Collect results
    let mut results = Vec::with_capacity(entries.len());
    while let Some(res) = join_set.join_next().await {
        match res {
            Ok(result) => results.push(result),
            Err(join_err) => {
                // Task panicked - convert to error
                let err = crate::Error::Bundler(vec![crate::diagnostics::ExtractedDiagnostic {
                    kind: crate::diagnostics::DiagnosticKind::Other("PanicDuringBuild".to_string()),
                    severity: crate::diagnostics::DiagnosticSeverity::Error,
                    message: format!("Build task panicked: {}", join_err),
                    file: None,
                    line: None,
                    column: None,
                    help: Some("This is a bug in fob. Please report it.".to_string()),
                    context: None,
                    error_chain: Vec::new(),
                }]);
                results.push(("unknown".to_string(), Err(err)));
            }
        }
    }

    results
}

/// Sequential fallback for WASM (single-threaded).
#[cfg(target_family = "wasm")]
async fn execute_builds_concurrent(
    options: &BuildOptions,
    entries: &[String],
) -> Vec<(String, Result<AnalyzedBundle>)> {
    let mut results = Vec::with_capacity(entries.len());
    for entry in entries {
        let result = build_single_component(options, entry).await;
        results.push((entry.clone(), result));
    }
    results
}

/// Merge build results in original entry order for deterministic output.
fn merge_build_results(
    results: Vec<(String, Result<AnalyzedBundle>)>,
    original_order: &[String],
) -> Result<BuildResult> {
    // Convert to map for O(1) lookup
    let mut results_map: FxHashMap<String, Result<AnalyzedBundle>> = results.into_iter().collect();

    let mut bundles = FxHashMap::default();
    let merged_graph = fob_graph::ModuleGraph::new()?;
    let mut all_entry_points = Vec::new();
    let mut all_warnings = Vec::new();
    let mut all_errors = Vec::new();
    let mut build_errors = Vec::new();
    let mut first_cache = None;
    let mut first_trace = None;
    let mut first_asset_registry = None;

    // Process in original order for determinism
    for entry in original_order {
        let name = entry_to_name(entry);

        match results_map.remove(entry) {
            Some(Ok(analyzed)) => {
                // Merge this component's graph into the accumulated graph
                let modules = analyzed.analysis.graph.modules()?;
                let entry_points_set: std::collections::HashSet<_> = analyzed
                    .analysis
                    .graph
                    .entry_points()?
                    .into_iter()
                    .collect();

                for module in modules {
                    let module_id = module.id.clone();
                    merged_graph.add_module(module)?;
                    if entry_points_set.contains(&module_id) {
                        merged_graph.add_entry_point(module_id.clone())?;
                    }

                    let deps = analyzed.analysis.graph.dependencies(&module_id)?;
                    for dep in deps {
                        merged_graph.add_dependency(module_id.clone(), dep)?;
                    }
                }

                all_entry_points.extend(analyzed.analysis.entry_points);
                all_warnings.extend(analyzed.analysis.warnings);
                all_errors.extend(analyzed.analysis.errors);

                if first_cache.is_none() {
                    first_cache = Some(analyzed.cache);
                }
                if first_trace.is_none() {
                    first_trace = Some(analyzed.trace);
                }
                if first_asset_registry.is_none() {
                    first_asset_registry = analyzed.asset_registry;
                }

                bundles.insert(name, analyzed.bundle);
            }
            Some(Err(e)) => {
                build_errors.push(format!("{}: {}", entry, e));
            }
            None => {
                build_errors.push(format!("{}: missing result", entry));
            }
        }
    }

    // Report collected errors
    if !build_errors.is_empty() {
        return Err(crate::Error::InvalidConfig(format!(
            "Build failed for {} entries:\n{}",
            build_errors.len(),
            build_errors.join("\n")
        )));
    }

    let stats = fob_graph::analysis::stats::compute_stats(&merged_graph)?;
    let symbol_stats = merged_graph.symbol_statistics()?;
    let analysis = fob_graph::AnalysisResult {
        graph: merged_graph,
        entry_points: all_entry_points,
        warnings: all_warnings,
        errors: all_errors,
        stats,
        symbol_stats,
    };

    Ok(BuildResult {
        output: BuildOutput::Multiple(bundles),
        analysis,
        cache: first_cache.unwrap_or_default(),
        trace: first_trace.unwrap_or_default(),
        asset_registry: first_asset_registry,
    })
}

/// Build a single component independently.
async fn build_single_component(options: &BuildOptions, entry: &str) -> Result<AnalyzedBundle> {
    let rolldown_options = configure_rolldown_options(options);

    let plan = BundlePlan {
        entries: vec![EntrySpec {
            name: None,
            import: entry.to_string(),
        }],
        options: rolldown_options,
        plugins: Vec::new(),
        cwd: options.cwd.clone(),
        virtual_files: options.virtual_files.clone(),
        runtime: options.runtime.clone(),
        cache: options.cache.clone(),
        incremental: options.incremental.clone(),
    };

    execute_bundle(plan).await
}

/// Configure Rolldown options from BuildOptions.
fn configure_rolldown_options(options: &BuildOptions) -> BundlerOptions {
    let mut rolldown_options = BundlerOptions {
        format: Some(options.format),
        sourcemap: options.sourcemap,
        ..Default::default()
    };

    // External packages configuration
    rolldown_options.external = Some(match &options.external {
        ExternalConfig::None => {
            // Bundle everything: no externals
            IsExternal::from(vec![])
        }
        ExternalConfig::List(packages) => {
            // Externalize specific packages
            IsExternal::from(packages.clone())
        }
        ExternalConfig::FromManifest(_path) => {
            // Externalize dependencies from package.json
            // TODO: Read package.json and extract dependencies/peerDependencies
            // For now, fall back to externalizing all bare imports
            IsExternal::from(vec!["^[^./]".to_string()])
        }
    });

    // Globals for IIFE/UMD
    if !options.globals.is_empty() {
        rolldown_options.globals = Some(GlobalsOutputOption::from(options.globals.clone()));
    }

    // Minification
    if let Some(level_str) = &options.minify_level {
        if let Ok(level) = MinifyLevel::parse(level_str) {
            rolldown_options.minify = level.to_rolldown_options();
        }
    }

    // Platform
    rolldown_options.platform = Some(options.platform);

    // Decorator transformation
    if let Some(decorator) = &options.decorator {
        let transform = rolldown_common::BundlerTransformOptions {
            decorator: Some(decorator.clone()),
            ..Default::default()
        };
        rolldown_options.transform = Some(transform);
    }

    // Module resolution
    let conditions = match options.platform {
        Platform::Browser => ExportConditions::browser(),
        Platform::Node => ExportConditions::node(),
        _ => ExportConditions::browser(),
    };
    rolldown_options.resolve = Some(configure_resolution(
        options.cwd.as_ref(),
        &options.path_aliases,
        &conditions,
    ));

    rolldown_options
}

/// Configure module resolution options.
fn configure_resolution(
    cwd: Option<&PathBuf>,
    path_aliases: &FxHashMap<String, String>,
    conditions: &ExportConditions,
) -> ResolveOptions {
    let modules = if let Some(cwd_path) = cwd {
        let mut paths = vec![];
        let mut current = cwd_path.as_path();

        loop {
            paths.push(current.join("node_modules").to_string_lossy().to_string());

            match current.parent() {
                Some(parent) => current = parent,
                None => break,
            }
        }

        paths.push("node_modules".to_string());
        paths
    } else {
        vec!["node_modules".to_string()]
    };

    let aliases = if !path_aliases.is_empty() {
        Some(convert_aliases_to_absolute(path_aliases, cwd))
    } else {
        None
    };

    let main_fields = if conditions.contains("node") {
        vec!["module".to_string(), "main".to_string()]
    } else {
        vec![
            "browser".to_string(),
            "module".to_string(),
            "main".to_string(),
        ]
    };

    ResolveOptions {
        alias: aliases,
        main_fields: Some(main_fields),
        condition_names: Some(conditions.to_vec()),
        extensions: Some(vec![
            ".js".to_string(),
            ".json".to_string(),
            ".mjs".to_string(),
            ".ts".to_string(),
            ".tsx".to_string(),
            ".mdx".to_string(),
        ]),
        modules: Some(modules),
        symlinks: Some(true),
        ..Default::default()
    }
}

/// Convert path aliases to absolute paths.
fn convert_aliases_to_absolute(
    aliases: &FxHashMap<String, String>,
    cwd: Option<&PathBuf>,
) -> Vec<(String, Vec<Option<String>>)> {
    let base_dir = cwd
        .cloned()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    aliases
        .iter()
        .map(|(alias, target)| {
            let target_path = Path::new(target);
            let absolute_target = if target_path.is_absolute() {
                target_path.to_path_buf()
            } else {
                base_dir.join(target_path)
            };

            (
                alias.clone(),
                vec![Some(absolute_target.to_string_lossy().to_string())],
            )
        })
        .collect()
}

/// Configure DTS generation plugin if enabled.
#[cfg(feature = "dts-generation")]
fn configure_dts_plugin(
    dts_opts: &crate::builders::unified::DtsOptions,
    entries: &[EntrySpec],
) -> Option<crate::SharedPluginable> {
    let should_emit = dts_opts
        .emit
        .unwrap_or_else(|| entries.iter().any(|e| is_typescript_entry(&e.import)));

    if should_emit {
        let plugin = DtsEmitPlugin::new(
            dts_opts.strip_internal,
            dts_opts.sourcemap,
            dts_opts.outdir.clone(),
        );
        Some(Arc::new(plugin))
    } else {
        None
    }
}

/// Helper function to detect if an entry is a TypeScript file.
#[cfg(feature = "dts-generation")]
fn is_typescript_entry(entry: &str) -> bool {
    Path::new(entry)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext, "ts" | "tsx" | "mts" | "cts"))
        .unwrap_or(false)
}

/// Convert an AnalyzedBundle to a BuildResult.
fn build_result_from_analyzed<F>(analyzed: AnalyzedBundle, wrapper: F) -> BuildResult
where
    F: FnOnce(crate::BundleOutput) -> BuildOutput,
{
    BuildResult {
        output: wrapper(analyzed.bundle),
        analysis: analyzed.analysis,
        cache: analyzed.cache,
        trace: analyzed.trace,
        asset_registry: analyzed.asset_registry,
    }
}

/// Extract a name from an entry path for use as a key.
fn entry_to_name(entry: &str) -> String {
    Path::new(entry)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("index")
        .to_string()
}

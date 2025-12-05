//! Build execution and dispatch logic.
//!
//! This module contains the internal implementation that dispatches
//! build operations to the appropriate execution path based on the
//! BuildOptions configuration.

use rolldown::{BundlerOptions, GlobalsOutputOption, IsExternal, Platform, ResolveOptions};
use rustc_hash::FxHashMap;
use std::path::{Path, PathBuf};

use crate::Result;
use crate::analysis::AnalyzedBundle;
use crate::builders::common::{BundlePlan, EntrySpec, execute_bundle};
use crate::builders::unified::{BuildOptions, BuildOutput, BuildResult, EntryPoints, MinifyLevel};
use crate::target::ExportConditions;

#[cfg(feature = "dts-generation")]
use std::sync::Arc;

#[cfg(feature = "dts-generation")]
use crate::DtsEmitPlugin;

/// Execute a build with the given options.
///
/// Dispatches to the appropriate build mode based on configuration.
pub async fn execute_build(options: BuildOptions) -> Result<BuildResult> {
    match (&options.entry, options.bundle, options.splitting) {
        // Library mode: don't bundle dependencies
        (EntryPoints::Single(_), false, _) => execute_library_build(options).await,

        // Components mode: Multiple entries, each bundled independently
        (EntryPoints::Multiple(_) | EntryPoints::Named(_), true, false) => {
            execute_components_build(options).await
        }

        // App mode: Multiple entries with code splitting
        (EntryPoints::Multiple(_) | EntryPoints::Named(_), true, true) => {
            execute_app_build(options).await
        }

        // Single entry, bundled (standalone build)
        (EntryPoints::Single(_), true, _) => execute_single_bundle(options).await,

        // Multiple entries without bundling (multiple libraries)
        (EntryPoints::Multiple(_) | EntryPoints::Named(_), false, _) => {
            execute_multiple_libraries(options).await
        }
    }
}

/// Execute a library build (bundle: false, single entry).
async fn execute_library_build(options: BuildOptions) -> Result<BuildResult> {
    let entry = match &options.entry {
        EntryPoints::Single(e) => e,
        _ => unreachable!("execute_library_build called with non-single entry"),
    };

    let rolldown_options = configure_rolldown_options(&options);

    // Library mode: externalize all dependencies
    // Don't set external here - Rolldown will handle it based on resolve failure
    // when bundle: false, imports that can't be resolved locally become external

    // Add DTS plugin if enabled
    #[cfg(feature = "dts-generation")]
    let plugins = if let Some(dts_opts) = &options.dts {
        let mut plugins = options.plugins.clone();
        if let Some(plugin) = configure_dts_plugin(dts_opts, entry) {
            plugins.push(plugin);
        }
        plugins
    } else {
        options.plugins.clone()
    };

    #[cfg(not(feature = "dts-generation"))]
    let plugins = options.plugins.clone();

    let plan = BundlePlan {
        entries: vec![EntrySpec {
            name: None,
            import: entry.clone(),
        }],
        options: rolldown_options,
        plugins,
        cwd: options.cwd.clone(),
        virtual_files: options.virtual_files.clone(),
        runtime: options.runtime.clone(),
    };

    let analyzed = execute_bundle(plan).await?;
    Ok(build_result_from_analyzed(analyzed, BuildOutput::Single))
}

/// Execute a components build (multiple independent bundles).
async fn execute_components_build(options: BuildOptions) -> Result<BuildResult> {
    let entries = match &options.entry {
        EntryPoints::Multiple(v) => v,
        EntryPoints::Named(m) => &m.values().cloned().collect::<Vec<_>>(),
        _ => unreachable!("execute_components_build called with single entry"),
    };

    let mut bundles = FxHashMap::default();
    let merged_graph = fob_graph::ModuleGraph::new()?;
    let mut all_entry_points = Vec::new();
    let mut all_warnings = Vec::new();
    let mut all_errors = Vec::new();
    let mut first_cache = None;
    let mut first_trace = None;
    let mut first_asset_registry = None;

    for entry in entries {
        let analyzed = build_single_component(&options, entry).await?;
        let name = entry_to_name(entry);

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
            // Add entry points from this component
            if entry_points_set.contains(&module_id) {
                merged_graph.add_entry_point(module_id.clone())?;
            }

            // Merge dependencies for this module
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
        plugins: options.plugins.clone(),
        cwd: options.cwd.clone(),
        virtual_files: options.virtual_files.clone(),
        runtime: options.runtime.clone(),
    };

    execute_bundle(plan).await
}

/// Execute an app build with code splitting.
async fn execute_app_build(options: BuildOptions) -> Result<BuildResult> {
    let entries = match &options.entry {
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
        _ => unreachable!("execute_app_build called with single entry"),
    };

    let mut rolldown_options = configure_rolldown_options(&options);
    rolldown_options.advanced_chunks = Some(rolldown::AdvancedChunksOptions {
        min_size: Some(20000.0),                // 20KB minimum chunk size
        min_share_count: Some(2),               // Shared by at least 2 chunks
        max_size: None,                         // No maximum chunk size limit
        min_module_size: None,                  // No minimum module size for splitting
        max_module_size: None,                  // No maximum module size limit
        include_dependencies_recursively: None, // Use default behavior
        groups: Some(vec![]),                   // No custom chunk groups
    });

    let plan = BundlePlan {
        entries,
        options: rolldown_options,
        plugins: options.plugins.clone(),
        cwd: options.cwd.clone(),
        virtual_files: options.virtual_files.clone(),
        runtime: options.runtime.clone(),
    };

    let analyzed = execute_bundle(plan).await?;
    Ok(build_result_from_analyzed(analyzed, BuildOutput::Single))
}

/// Execute a single bundled entry (not library mode).
async fn execute_single_bundle(options: BuildOptions) -> Result<BuildResult> {
    let entry = match &options.entry {
        EntryPoints::Single(e) => e,
        _ => unreachable!("execute_single_bundle called with non-single entry"),
    };

    let rolldown_options = configure_rolldown_options(&options);

    let plan = BundlePlan {
        entries: vec![EntrySpec {
            name: None,
            import: entry.clone(),
        }],
        options: rolldown_options,
        plugins: options.plugins.clone(),
        cwd: options.cwd.clone(),
        virtual_files: options.virtual_files.clone(),
        runtime: options.runtime.clone(),
    };

    let analyzed = execute_bundle(plan).await?;
    Ok(build_result_from_analyzed(analyzed, BuildOutput::Single))
}

/// Execute multiple independent library builds.
async fn execute_multiple_libraries(options: BuildOptions) -> Result<BuildResult> {
    // For now, treat this similar to components but without bundling
    execute_components_build(options).await
}

/// Configure Rolldown options from BuildOptions.
fn configure_rolldown_options(options: &BuildOptions) -> BundlerOptions {
    let mut rolldown_options = BundlerOptions {
        format: Some(options.format),
        sourcemap: options.sourcemap,
        ..Default::default()
    };

    // External packages configuration
    // IMPORTANT: Always set external explicitly - Rolldown treats None differently from Some([])
    if options.bundle {
        // Bundle mode: only externalize what user specified (empty = bundle everything)
        rolldown_options.external = Some(IsExternal::from(options.external.clone()));
    } else {
        // Library mode: externalize all bare imports + user additions
        let mut patterns = vec!["^[^./]".to_string()];
        patterns.extend(options.external.clone());
        rolldown_options.external = Some(IsExternal::from(patterns));
    }

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

    // Module resolution with absolute paths for pnpm/monorepo support
    // Map Platform to ExportConditions (bridge for BuildOptions compatibility)
    // Note: BuildConfig uses DeploymentTarget directly for better control
    let conditions = match options.platform {
        Platform::Browser => ExportConditions::browser(),
        Platform::Node => ExportConditions::node(),
        // Default to browser for other platforms
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
///
/// Uses multiple resolution paths to properly handle pnpm symlinks,
/// monorepos, and nested package structures. Also configures path aliases
/// for import resolution (e.g., "@/" -> "./src/").
///
/// Uses export conditions from the deployment target to determine which
/// package.json export conditions to try during module resolution.
/// This fixes platform-specific resolution issues (e.g., Vercel SSR using
/// Node.js conditions instead of browser conditions).
fn configure_resolution(
    cwd: Option<&PathBuf>,
    path_aliases: &FxHashMap<String, String>,
    conditions: &ExportConditions,
) -> ResolveOptions {
    // Use multiple paths for modules to handle different package manager layouts
    // Walk up parent directories to find node_modules (matches Node.js resolution)
    // This supports pnpm/yarn/npm workspaces and monorepos
    let modules = if let Some(cwd_path) = cwd {
        let mut paths = vec![];
        let mut current = cwd_path.as_path();

        // Search current and all parent directories for node_modules
        loop {
            paths.push(current.join("node_modules").to_string_lossy().to_string());

            match current.parent() {
                Some(parent) => current = parent,
                None => break,
            }
        }

        paths.push("node_modules".to_string()); // Relative fallback
        paths
    } else {
        vec!["node_modules".to_string()]
    };

    // Convert path aliases to absolute paths for Rolldown
    let aliases = if !path_aliases.is_empty() {
        Some(convert_aliases_to_absolute(path_aliases, cwd))
    } else {
        None
    };

    // Determine main_fields based on conditions
    // For Node.js, prefer "main" over "browser"
    // For browser/edge, prefer "browser" over "main"
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
        symlinks: Some(true), // Follow symlinks (important for pnpm)
        ..Default::default()
    }
}

/// Convert path aliases to absolute paths.
///
/// Path aliases from tsconfig.json or user configuration are typically relative.
/// Rolldown requires absolute paths for alias resolution. This function converts
/// relative paths to absolute paths based on the current working directory.
///
/// Example:
/// - Input: `{"@": "./src", "~": "./lib"}`
/// - CWD: `/Users/fox/project`
/// - Output: `{"@": "/Users/fox/project/src", "~": "/Users/fox/project/lib"}`
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

            // Rolldown expects Vec<Option<String>> for each alias to support multiple targets
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
    entry: &str,
) -> Option<crate::SharedPluginable> {
    // Auto-detect TypeScript if emit is None
    let should_emit = dts_opts.emit.unwrap_or_else(|| is_typescript_entry(entry));

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

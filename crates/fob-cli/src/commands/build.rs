//! Build command implementation.
//!
//! This module implements the `fob build` command, which bundles JavaScript/TypeScript
//! files using the fob-core library.

use crate::cli::BuildArgs;
use crate::commands::utils;
use crate::config::{DocsFormat, FobConfig};
use crate::error::{BuildError, CliError, Result};
use crate::ui;
use fob_core::{DocsEmitPlugin, DocsEmitPluginOptions, DocsPluginOutputFormat, NativeRuntime};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tracing;

/// Execute the build command.
///
/// # Build Process
///
/// 1. Load and validate configuration (CLI > Env > File > Defaults)
/// 2. Clean output directory if requested
/// 3. Validate entry points
/// 4. Execute build with progress tracking
/// 5. Write output files
/// 6. Display build summary
///
/// # Arguments
///
/// * `args` - Parsed command-line arguments
///
/// # Errors
///
/// Returns errors for:
/// - Invalid configuration
/// - Missing entry points
/// - Build failures
/// - File system errors
pub async fn execute(args: BuildArgs) -> Result<()> {
    let start_time = Instant::now();

    // Step 1: Load configuration
    ui::info("Loading configuration...");
    let config = FobConfig::load(&args, None)?;
    config.validate()?;

    // Resolve project root using smart auto-detection
    let cwd = utils::resolve_project_root(
        config.cwd.as_deref(),                    // explicit --cwd flag
        config.entry.first().map(|s| s.as_str()), // first entry point
    )?;

    // Step 2: Clean output if requested
    if config.clean {
        let out_dir = utils::resolve_path(&config.out_dir, &cwd);
        ui::info(&format!("Cleaning output directory: {}", out_dir.display()));
        utils::clean_output_dir(&out_dir)?;
    } else {
        let out_dir = utils::resolve_path(&config.out_dir, &cwd);
        utils::ensure_output_dir(&out_dir)?;
    }

    // Step 3: Validate entry points
    if config.entry.is_empty() {
        return Err(CliError::InvalidArgument(
            "At least one entry point is required".to_string(),
        ));
    }

    for entry in &config.entry {
        let entry_path = utils::resolve_path(std::path::Path::new(entry), &cwd);
        utils::validate_entry(&entry_path)?;
    }

    // Step 3.5: Write back enhanced documentation to source files (if requested)
    #[cfg(feature = "llm-docs")]
    if config.docs_enhance && config.docs_write_back {
        perform_docs_writeback(&config, &cwd).await?;
    }

    // Step 4: Execute build
    build(&config, &cwd).await?;

    let duration = start_time.elapsed();
    ui::success(&format!(
        "Build completed in {}",
        ui::format_duration(duration)
    ));

    Ok(())
}

/// Unified build function that returns the BuildResult.
///
/// Builds based on configuration, not mode detection:
/// - Single entry → BuildOptions::new()
/// - Multiple entries → BuildOptions::new_multiple()
/// - Applies bundle, splitting, platform, etc. from config
///
/// This version returns the BuildResult for dev server use.
pub(crate) async fn build_with_result(config: &FobConfig, cwd: &std::path::Path) -> Result<fob_core::BuildResult> {
    validate_output_dir(&config.out_dir, cwd)?;

    // Display build info
    if config.entry.len() == 1 {
        ui::info(&format!("Building: {}", config.entry[0]));
    } else {
        ui::info(&format!("Building {} entries...", config.entry.len()));
        for entry in &config.entry {
            ui::info(&format!("  - {}", entry));
        }
    }
    ui::info(&format!("Bundle: {}", config.bundle));
    ui::info(&format!("Format: {:?}", config.format));
    if config.splitting {
        ui::info("Code splitting: enabled");
    }
    ui::info(&format!("Output: {}", config.out_dir.display()));

    // Create builder based on entry count
    let mut builder = if config.entry.len() == 1 {
        fob_core::BuildOptions::new(&config.entry[0])
    } else {
        fob_core::BuildOptions::new_multiple(&config.entry)
    };

    // Apply configuration
    builder = builder
        .bundle(config.bundle)
        .format(convert_format(config.format))
        .minify(config.minify)
        .platform(convert_platform(config.platform))
        .splitting(config.splitting)
        .cwd(cwd)
        .runtime(Arc::new(NativeRuntime));

    // Sourcemap
    if let Some(sourcemap_mode) = config.sourcemap {
        builder = match sourcemap_mode {
            crate::config::SourceMapMode::Inline => builder.sourcemap_inline(),
            crate::config::SourceMapMode::External => builder.sourcemap(true),
            crate::config::SourceMapMode::Hidden => builder.sourcemap_hidden(),
        };
    }

    // Externals
    if !config.external.is_empty() {
        builder = builder.external(&config.external);
    }

    // Global name for IIFE
    if let Some(ref name) = config.global_name {
        builder = builder.globals_map([("__self__".to_string(), name.clone())]);
    }

    // Apply docs plugin if requested
    builder = apply_docs_plugin(builder, config);

    // TypeScript declarations
    #[cfg(feature = "dts-generation")]
    {
        if config.dts {
            builder = builder.emit_dts(true);
        }
    }

    // Build
    let result = builder
        .build()
        .await
        .map_err(|e| CliError::Build(BuildError::Custom(format!("Build failed: {}", e))))?;

    // Write output (force overwrite for build command)
    result.write_to_force(&config.out_dir).map_err(|e| {
        CliError::Build(BuildError::Custom(format!("Failed to write output: {}", e)))
    })?;

    ui::success(&format!("Built to {}", config.out_dir.display()));

    Ok(result)
}

/// Unified build function that applies configuration directly.
///
/// Builds based on configuration, not mode detection:
/// - Single entry → BuildOptions::new()
/// - Multiple entries → BuildOptions::new_multiple()
/// - Applies bundle, splitting, platform, etc. from config
pub(crate) async fn build(config: &FobConfig, cwd: &std::path::Path) -> Result<()> {
    build_with_result(config, cwd).await?;
    Ok(())
}

/// Validates that the output directory is safe to write to.
///
/// # Security
///
/// Prevents writing to dangerous locations that could corrupt the system:
/// - Root directories (/, /usr, /etc, etc.)
/// - System directories
/// - Paths outside the project tree
///
/// # Errors
///
/// Returns `OutputNotWritable` if the directory is unsafe.
fn validate_output_dir(out_dir: &Path, cwd: &Path) -> Result<()> {
    let canonical_out = if out_dir.exists() {
        out_dir.canonicalize()?
    } else {
        let parent = out_dir
            .parent()
            .ok_or_else(|| CliError::Build(BuildError::OutputNotWritable(out_dir.to_path_buf())))?;
        parent.canonicalize()?.join(out_dir.file_name().unwrap())
    };

    let canonical_cwd = cwd.canonicalize()?;

    let is_within_project = canonical_out.starts_with(&canonical_cwd);
    let is_sibling = canonical_out
        .parent()
        .and_then(|p| canonical_cwd.parent().map(|c| p == c))
        .unwrap_or(false);

    if !is_within_project && !is_sibling {
        return Err(CliError::Build(BuildError::OutputNotWritable(out_dir.to_path_buf())).into());
    }

    const DANGEROUS_PATHS: &[&str] = &[
        "/bin",
        "/boot",
        "/dev",
        "/etc",
        "/lib",
        "/lib64",
        "/proc",
        "/root",
        "/sbin",
        "/sys",
        "/usr/bin",
        "/usr/lib",
        "/usr/sbin",
        "/var/log",
    ];

    let out_str = canonical_out.to_string_lossy();
    for dangerous in DANGEROUS_PATHS {
        if out_str.starts_with(dangerous) {
            return Err(CliError::Build(BuildError::Custom(format!(
                "Refusing to write to system directory: {}",
                out_str
            )))
            .into());
        }
    }

    if out_str == "/" {
        return Err(CliError::Build(BuildError::Custom(
            "Refusing to write to root directory".to_string(),
        ))
        .into());
    }

    Ok(())
}

/// Convert CLI format enum to fob-core OutputFormat
fn convert_format(format: crate::config::Format) -> fob_core::OutputFormat {
    match format {
        crate::config::Format::Esm => fob_core::OutputFormat::Esm,
        crate::config::Format::Cjs => fob_core::OutputFormat::Cjs,
        crate::config::Format::Iife => fob_core::OutputFormat::Iife,
    }
}

/// Convert CLI platform enum to fob-core Platform
fn convert_platform(platform: crate::config::Platform) -> fob_core::Platform {
    match platform {
        crate::config::Platform::Browser => fob_core::Platform::Browser,
        crate::config::Platform::Node => fob_core::Platform::Node,
    }
}

fn apply_docs_plugin(
    builder: fob_core::BuildOptions,
    config: &FobConfig,
) -> fob_core::BuildOptions {
    if !config.docs {
        return builder;
    }

    let mut options = DocsEmitPluginOptions::default();
    if let Some(dir) = &config.docs_dir {
        options.output_dir = Some(dir.clone());
    }
    options.include_internal = config.docs_include_internal;
    let format = config.docs_format.unwrap_or(DocsFormat::Markdown);
    options.output_format = convert_docs_format(format);

    // Add LLM configuration if enhancement is enabled
    #[cfg(feature = "llm-docs")]
    if config.docs_enhance {
        if let Some(ref llm_config_data) = config.docs_llm {
            use fob_core::{EnhancementMode, LlmConfig};

            let enhancement_mode = match llm_config_data.mode.as_str() {
                "incomplete" => EnhancementMode::Incomplete,
                "all" => EnhancementMode::All,
                _ => EnhancementMode::Missing,
            };

            let mut llm_config = LlmConfig::default()
                .with_model(&llm_config_data.model)
                .with_mode(enhancement_mode);

            if !llm_config_data.cache {
                llm_config = llm_config.without_cache();
            }

            if let Some(ref url) = llm_config_data.url {
                llm_config = llm_config.with_url(url);
            }

            options.llm_config = Some(llm_config);
        }
    }

    let plugin = DocsEmitPlugin::new(options);
    builder.plugin(fob_core::plugin(plugin))
}

fn convert_docs_format(format: DocsFormat) -> DocsPluginOutputFormat {
    match format {
        DocsFormat::Markdown => DocsPluginOutputFormat::Markdown,
        DocsFormat::Json => DocsPluginOutputFormat::Json,
        DocsFormat::Both => DocsPluginOutputFormat::Both,
    }
}

/// Performs documentation extraction, LLM enhancement, and writeback to source files.
#[cfg(feature = "llm-docs")]
async fn perform_docs_writeback(config: &FobConfig, cwd: &std::path::Path) -> Result<()> {
    use fob_core::{EnhancementMode, LlmConfig, LlmEnhancer};
    use fob_docs::{DocsExtractor, ExtractOptions, MergeStrategy, DocsWriteback};

    ui::info("Extracting documentation from source files...");

    // Extract documentation from all entry files
    let options = ExtractOptions {
        include_internal: config.docs_include_internal,
    };
    let extractor = DocsExtractor::new(options);
    let mut all_docs = vec![];

    for entry in &config.entry {
        let entry_path = utils::resolve_path(std::path::Path::new(entry), cwd);

        match extractor.extract_from_path(&entry_path) {
            Ok(module_doc) => {
                all_docs.push(module_doc);
            }
            Err(e) => {
                tracing::warn!("Failed to extract docs from {}: {}", entry, e);
            }
        }
    }

    if all_docs.is_empty() {
        ui::info("No documentation found in source files.");
        return Ok(());
    }

    let mut documentation = fob_core::Documentation {
        modules: all_docs,
    };

    ui::info(&format!("Extracted documentation from {} modules", documentation.modules.len()));

    // Enhance with LLM
    if let Some(ref llm_config_data) = config.docs_llm {
        let enhancement_mode = match llm_config_data.mode.as_str() {
            "incomplete" => EnhancementMode::Incomplete,
            "all" => EnhancementMode::All,
            _ => EnhancementMode::Missing,
        };

        let mut llm_config = LlmConfig::default()
            .with_model(&llm_config_data.model)
            .with_mode(enhancement_mode);

        if !llm_config_data.cache {
            llm_config = llm_config.without_cache();
        }

        if let Some(ref url) = llm_config_data.url {
            llm_config = llm_config.with_url(url);
        }

        ui::info(&format!("Enhancing documentation with LLM (model: {})...", llm_config_data.model));

        let enhancer = match LlmEnhancer::new(llm_config).await {
            Ok(enhancer) => enhancer,
            Err(e) => {
                tracing::warn!("Failed to initialize LLM enhancer: {}", e);
                ui::info("Continuing without LLM enhancement...");
                // Continue with original documentation
                return perform_writeback(config, documentation);
            }
        };

        let total_symbols: usize =
            documentation.modules.iter().map(|m| m.symbols.len()).sum();

        documentation = match enhancer
            .enhance_documentation(documentation, |current, total| {
                if current % 5 == 0 || current == total {
                    tracing::debug!("[LLM] Progress: {}/{}", current, total);
                }
            })
            .await
        {
            Ok(enhanced) => {
                ui::info("LLM enhancement complete!");
                enhanced
            }
            Err(e) => {
                tracing::warn!("LLM enhancement failed: {}", e);
                ui::info("Unable to continue with original documentation after enhancement failure.");
                // Since documentation was moved, we can't recover
                return Ok(());
            }
        };
    }

    perform_writeback(config, documentation)
}

/// Helper function to perform the actual writeback.
#[cfg(feature = "llm-docs")]
fn perform_writeback(config: &FobConfig, documentation: fob_core::Documentation) -> Result<()> {
    use fob_docs::{MergeStrategy, DocsWriteback};

    ui::info("Writing enhanced documentation back to source files...");

    let merge_strategy = match config.docs_merge_strategy.as_deref() {
        Some("replace") => MergeStrategy::Replace,
        Some("skip") => MergeStrategy::Skip,
        _ => MergeStrategy::Merge, // default
    };

    let writeback = DocsWriteback::new(
        !config.docs_no_backup,  // create_backups
        true,                    // skip_node_modules
        merge_strategy,
    );

    match writeback.write_documentation(&documentation) {
        Ok(report) => {
            ui::success("Documentation writeback complete!");
            ui::info(&format!("  Files modified: {}", report.files_modified));
            ui::info(&format!("  Symbols updated: {}", report.symbols_updated));
            if report.files_backed_up > 0 {
                ui::info(&format!("  Backups created: {} (.bak files)", report.files_backed_up));
            }
            if report.symbols_skipped > 0 {
                ui::info(&format!("  Symbols skipped: {}", report.symbols_skipped));
            }
            if !report.errors.is_empty() {
                tracing::warn!("Errors during writeback: {}", report.errors.len());
                for error in &report.errors {
                    tracing::warn!("  - {}", error);
                }
            }
        }
        Err(e) => {
            return Err(CliError::Build(BuildError::Custom(format!("Documentation writeback failed: {}", e))));
        }
    }

    Ok(())
}

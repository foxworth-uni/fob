//! Build command implementation.
//!
//! This module implements the `fob build` command, which bundles JavaScript/TypeScript
//! files using the fob-core library.

use crate::cli::BuildArgs;
use crate::commands::utils;
use crate::config::FobConfig;
use crate::error::{BuildError, CliError, Result};
use crate::ui;
use fob_bundler::NativeRuntime;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

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
        config.entry.first().map(String::as_str), // first entry point
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
/// Builds based on configuration using the new composable primitives API:
/// - bundle=false → new_library (externalize deps)
/// - splitting=true with multiple entries → new_app (code splitting)
/// - Otherwise → new_multiple (separate bundles) or new (single entry)
///
/// This version returns the BuildResult for dev server use.
pub(crate) async fn build_with_result(
    config: &FobConfig,
    cwd: &std::path::Path,
) -> Result<fob_bundler::BuildResult> {
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

    // Display mode info based on config
    let mode = if !config.bundle {
        "library (externalize deps)"
    } else if config.splitting && config.entry.len() > 1 {
        "app (code splitting)"
    } else if config.entry.len() > 1 {
        "components (separate bundles)"
    } else {
        "standalone"
    };
    ui::info(&format!("Mode: {}", mode));
    ui::info(&format!("Format: {:?}", config.format));
    ui::info(&format!("Output: {}", config.out_dir.display()));

    // Create builder based on config using new composable primitives
    // Map old config fields to new constructor patterns:
    // - bundle=false → externalize_from("package.json")
    // - splitting=true with multiple entries → bundle_together().with_code_splitting()
    // - multiple entries without splitting → bundle_separately()
    // - single entry with bundle=true → new (standard bundle)
    let mut builder = if !config.bundle {
        // Library mode: externalize dependencies
        if config.entry.len() == 1 {
            fob_bundler::BuildOptions::new(&config.entry[0]).externalize_from("package.json")
        } else {
            // Multiple library entries - use new_multiple with externalize
            fob_bundler::BuildOptions::new_multiple(&config.entry).externalize_from("package.json")
        }
    } else if config.splitting && config.entry.len() > 1 {
        // App mode: multiple entries with code splitting
        fob_bundler::BuildOptions::new_multiple(&config.entry)
            .bundle_together()
            .with_code_splitting()
    } else if config.entry.len() > 1 {
        // Components mode: multiple separate bundles
        fob_bundler::BuildOptions::new_multiple(&config.entry).bundle_separately()
    } else {
        // Standalone: single entry, full bundling
        fob_bundler::BuildOptions::new(&config.entry[0])
    };

    // Apply common configuration
    builder = builder
        .format(convert_format(config.format))
        .platform(convert_platform(config.platform))
        .cwd(cwd)
        .runtime(Arc::new(NativeRuntime));

    // Minification
    if config.minify {
        builder = builder.minify_level("identifiers");
    }

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
        builder = builder.externalize(&config.external);
    }

    // Global name for IIFE
    if let Some(ref name) = config.global_name {
        builder = builder.globals_map([("__self__".to_string(), name.clone())]);
    }

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
    let resolved_out_dir = utils::resolve_path(&config.out_dir, cwd);
    result.write_to_force(&resolved_out_dir).map_err(|e| {
        CliError::Build(BuildError::Custom(format!("Failed to write output: {}", e)))
    })?;

    ui::success(&format!("Built to {}", config.out_dir.display()));

    Ok(result)
}

/// Unified build function that applies configuration directly.
///
/// Builds based on configuration using composable primitives:
/// - bundle=false → library mode (externalize deps)
/// - splitting=true → app mode (code splitting)
/// - Otherwise → components or standalone mode
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
    let resolved_out_dir = utils::resolve_path(out_dir, cwd);
    let canonical_out = if resolved_out_dir.exists() {
        resolved_out_dir.canonicalize()?
    } else {
        let parent = resolved_out_dir.parent().ok_or_else(|| {
            CliError::Build(BuildError::OutputNotWritable(resolved_out_dir.clone()))
        })?;
        let filename = resolved_out_dir.file_name().ok_or_else(|| {
            CliError::Build(BuildError::OutputNotWritable(resolved_out_dir.clone()))
        })?;
        parent.canonicalize()?.join(filename)
    };

    let canonical_cwd = cwd.canonicalize()?;

    let is_within_project = canonical_out.starts_with(&canonical_cwd);
    let is_sibling = canonical_out
        .parent()
        .and_then(|p| canonical_cwd.parent().map(|c| p == c))
        .unwrap_or(false);

    if !is_within_project && !is_sibling {
        return Err(CliError::Build(BuildError::OutputNotWritable(
            resolved_out_dir,
        )));
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
            ))));
        }
    }

    if out_str == "/" {
        return Err(CliError::Build(BuildError::Custom(
            "Refusing to write to root directory".to_string(),
        )));
    }

    Ok(())
}

/// Convert CLI format enum to fob-bundler OutputFormat
fn convert_format(format: crate::config::Format) -> fob_bundler::OutputFormat {
    match format {
        crate::config::Format::Esm => fob_bundler::OutputFormat::Esm,
        crate::config::Format::Cjs => fob_bundler::OutputFormat::Cjs,
        crate::config::Format::Iife => fob_bundler::OutputFormat::Iife,
    }
}

/// Convert CLI platform enum to fob-bundler Platform
fn convert_platform(platform: crate::config::Platform) -> fob_bundler::Platform {
    match platform {
        crate::config::Platform::Browser => fob_bundler::Platform::Browser,
        crate::config::Platform::Node => fob_bundler::Platform::Node,
    }
}

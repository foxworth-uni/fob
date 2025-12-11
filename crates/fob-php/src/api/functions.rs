//! Standalone PHP functions
#![allow(dead_code)] // Functions exported via #[php_function] macro

use crate::RUNTIME;
use crate::api::config::BundleConfig;
use crate::api::utils::parse_format_normalized;
use crate::conversion::result::build_result_to_php_array;
use crate::core::bundler::CoreBundler;
use crate::error::bundler_error_to_php_exception;
use crate::types::LogLevel;
use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;

/// Initialize fob logging with specified level.
///
/// Call this once at application startup before any fob operations.
///
/// # Arguments
///
/// * `level` - Optional log level - "silent", "error", "warn", "info", or "debug" (default: "info")
#[php_function]
pub fn init_logging(level: Option<String>) -> PhpResult<()> {
    let log_level = level
        .as_ref()
        .and_then(|s| LogLevel::from_str(s))
        .unwrap_or_default();
    fob_bundler::init_logging(log_level.to_bundler_level());
    Ok(())
}

/// Initialize logging from RUST_LOG environment variable.
///
/// Falls back to Info level if RUST_LOG is not set or invalid.
#[php_function]
pub fn init_logging_from_env() -> PhpResult<()> {
    fob_bundler::init_logging_from_env();
    Ok(())
}

/// Quick helper to bundle a single entry.
///
/// Convenience function for simple bundling scenarios.
///
/// # Arguments
///
/// * `entry` - Entry file path
/// * `output_dir` - Output directory path
/// * `format` - Optional output format - "esm", "cjs", or "iife" (default: "esm")
///
/// # Returns
///
/// Array containing bundle result
#[php_function]
pub fn bundle_single(entry: String, output_dir: String, format: Option<String>) -> PhpResult<Zval> {
    // Use current working directory - PHP script should chdir if needed
    let cwd = std::env::current_dir()
        .ok()
        .map(|p| p.to_string_lossy().to_string());

    let output_format = format.as_ref().and_then(|f| parse_format_normalized(f));

    let config = BundleConfig {
        entries: vec![entry],
        output_dir: Some(output_dir),
        format: output_format,
        sourcemap: Some("external".to_string()),
        external: None,
        platform: None,
        minify: None,
        cwd,
        mdx: None, // Auto-detect from entry extension
        entry_mode: None,
        code_splitting: None,
        external_from_manifest: None,
    };

    let bundler = RUNTIME.block_on(async {
        CoreBundler::new(config).map_err(|e| PhpException::default(e.to_string()))
    })?;

    let result = RUNTIME
        .block_on(bundler.bundle())
        .map_err(bundler_error_to_php_exception)?;

    build_result_to_php_array(&result)
}

/// Get the bundler version.
///
/// # Returns
///
/// Version string (e.g., "0.3.0")
#[php_function]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

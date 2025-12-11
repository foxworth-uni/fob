//! Standalone NAPI functions

use crate::api::config::BundleConfig;
use crate::conversion::result::BundleResult;
use crate::core::bundler::CoreBundler;
use crate::types::parse_log_level;
use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Initialize fob logging with specified level
///
/// Call this once at application startup before any fob operations.
/// It is safe to call multiple times - only the first call takes effect.
///
/// @param level - "silent" | "error" | "warn" | "info" | "debug" (case-insensitive, default: "info")
///
/// @example
/// ```typescript
/// import { initLogging } from '@fob/native';
///
/// initLogging('info');
/// // or
/// initLogging('debug');
/// ```
#[napi]
pub fn init_logging(level: Option<String>) {
    let level = parse_log_level(level.as_deref());
    fob_bundler::init_logging(level);
}

/// Initialize logging from RUST_LOG environment variable
///
/// Falls back to Info level if RUST_LOG is not set or invalid.
/// Call this once at application startup before any fob operations.
///
/// @example
/// ```typescript
/// import { initLoggingFromEnv } from '@fob/native';
///
/// // Set RUST_LOG=fob=debug before running
/// initLoggingFromEnv();
/// ```
#[napi]
pub fn init_logging_from_env() {
    fob_bundler::init_logging_from_env();
}

/// Quick helper to bundle a single entry
///
/// @param format - "esm" | "cjs" | "iife" (case-insensitive, default: "esm")
#[napi]
pub async fn bundle_single(
    entry: String,
    output_dir: String,
    format: Option<String>,
) -> Result<BundleResult> {
    // Derive cwd from entry file's parent directory
    let entry_path = std::path::Path::new(&entry);
    let cwd = entry_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|p| p.to_string_lossy().to_string())
        });

    let config = BundleConfig {
        entries: vec![entry],
        output_dir: Some(output_dir),
        format,
        sourcemap: Some("external".to_string()),
        external: None,
        platform: None,
        minify: None,
        cwd,
        mdx: None, // Auto-detect from entry extension
        // Use defaults for composable primitives
        entry_mode: None,
        code_splitting: None,
        external_from_manifest: None,
        virtual_files: None,
    };

    let bundler = CoreBundler::new(config).map_err(|e| Error::from_reason(e.to_string()))?;
    bundler.bundle().await.map_err(|e| {
        let details = crate::error_mapper::map_bundler_error(&e);
        Error::from_reason(details.to_napi_json_string())
    })
}

/// Get the bundler version
#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

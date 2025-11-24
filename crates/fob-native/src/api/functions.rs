//! Standalone NAPI functions

use crate::api::config::BundleConfig;
use crate::conversion::result::BundleResult;
use crate::core::bundler::CoreBundler;
use crate::types::{OutputFormat, SourceMapMode};
use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Quick helper to bundle a single entry
#[napi]
pub async fn bundle_single(
    entry: String,
    output_dir: String,
    format: Option<OutputFormat>,
) -> Result<BundleResult> {
    // Derive cwd from entry file's parent directory
    let entry_path = std::path::Path::new(&entry);
    let cwd = entry_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .or_else(|| std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()));
    
    let config = BundleConfig {
        entries: vec![entry],
        output_dir: Some(output_dir),
        format,
        sourcemap: Some(SourceMapMode::External),
        cwd,
    };

    let bundler = CoreBundler::new(config)
        .map_err(|e| Error::from_reason(e.to_string()))?;
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


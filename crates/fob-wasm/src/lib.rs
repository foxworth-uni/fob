//! # fob-wasm: WASI Component Model Bindings for Fob
//!
//! Pre-configured task-based builders for WASI environments with MDX support.
//! Uses WASI Component Model with WIT interface definitions.

use fob_core::{plugin, BuildOptions, BunnyMdxPlugin, OutputFormat};
use std::fs;
use std::path::PathBuf;

wit_bindgen::generate!({
    world: "bundler",
    path: "./wit/bundler.wit",
});

use crate::exports::fob::bundler::bundle_api::{BundleConfig, BundleResult, Guest};

struct BundlerComponent;

impl Guest for BundlerComponent {
    fn bundle(config: BundleConfig) -> Result<BundleResult, String> {
        // Use tokio runtime to handle async operations
        // Component Model doesn't support async exports yet, so we block
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;

        // Convert WIT types to internal representation
        let config_internal = InternalBundleConfig {
            entries: config.entries,
            output_dir: config.output_dir,
            format: config.format,
            sourcemap: config.sourcemap,
        };

        let result = rt.block_on(bundle_internal(config_internal))
            .map_err(|e| e.to_string())?;

        // Convert back to WIT types
        Ok(BundleResult {
            assets_count: result.assets_count as u32,
            success: result.success,
            error: result.error,
        })
    }

    fn get_runtime_version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

/// Internal bundle config (matches WIT structure)
struct InternalBundleConfig {
    entries: Vec<String>,
    output_dir: String,
    format: Option<String>,
    sourcemap: Option<bool>,
}

/// Internal bundle result
struct InternalBundleResult {
    assets_count: usize,
    success: bool,
    error: Option<String>,
}

/// Internal bundle function (async)
async fn bundle_internal(
    config: InternalBundleConfig,
) -> Result<InternalBundleResult, Box<dyn std::error::Error>> {
    if config.entries.is_empty() {
        return Err("No entry points provided".into());
    }

    let format = match config.format.as_deref() {
        Some("esm") | None => OutputFormat::Esm,
        Some("cjs") => OutputFormat::Cjs,
        Some("iife") => OutputFormat::Iife,
        Some(f) => {
            return Err(format!("Unknown format: {}. Use 'esm', 'cjs', or 'iife'", f).into())
        }
    };

    let sourcemap = config.sourcemap.unwrap_or(false);
    let out_dir = PathBuf::from(&config.output_dir);

    fs::create_dir_all(&out_dir).map_err(|e| format!("Failed to create output dir: {}", e))?;

    if config.entries.len() == 1 {
        let entry = &config.entries[0];

        let result = BuildOptions::library(entry.clone())
            .plugin(plugin(BunnyMdxPlugin::new()))
            .format(format)
            .sourcemap(sourcemap)
            .build()
            .await
            .map_err(|e| format!("Bundle failed: {}", e))?;

        let bundle = result.output.as_single().expect("single bundle");
        for asset in bundle.assets.iter() {
            let path = out_dir.join(asset.filename());
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!("Failed to create directory {}: {}", parent.display(), e)
                })?;
            }
            fs::write(&path, asset.content_as_bytes())
                .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
        }

        Ok(InternalBundleResult {
            assets_count: bundle.assets.len(),
            success: true,
            error: None,
        })
    } else {
        let result = BuildOptions::components(config.entries.clone())
            .plugin(plugin(BunnyMdxPlugin::new()))
            .format(format)
            .sourcemap(sourcemap)
            .build()
            .await
            .map_err(|e| format!("Bundle failed: {}", e))?;

        let bundles = result.output.as_multiple().expect("multiple bundles");
        let mut assets = 0usize;
        for (_, bundle) in bundles.iter() {
            for asset in bundle.assets.iter() {
                let path = out_dir.join(asset.filename());
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        format!("Failed to create directory {}: {}", parent.display(), e)
                    })?;
                }
                fs::write(&path, asset.content_as_bytes())
                    .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
            }
            assets += bundle.assets.len();
        }

        Ok(InternalBundleResult {
            assets_count: assets,
            success: true,
            error: None,
        })
    }
}

export!(BundlerComponent);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn default_bundle_runs() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let entry = temp_dir.path().join("index.js");
        std::fs::write(&entry, "console.log('hello from wasm bundler');").unwrap();

        let config = InternalBundleConfig {
            entries: vec![entry.display().to_string()],
            output_dir: temp_dir.path().join("dist").display().to_string(),
            format: Some("esm".to_string()),
            sourcemap: Some(true),
        };

        let result = bundle_internal(config).await.unwrap();
        assert!(result.success);
        assert!(result.assets_count > 0);
    }
}

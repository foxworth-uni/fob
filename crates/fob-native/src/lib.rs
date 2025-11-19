#![deny(clippy::all)]

//! Native Node.js bindings for Fob bundler core

mod bundle_result;
mod runtime;

use bundle_result::BundleResult;
use fob_bundler::{BuildOptions, OutputFormat, Runtime};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use runtime::NativeRuntime;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

/// Bundle configuration
#[napi(object)]
pub struct BundleConfig {
    /// Entry points to bundle
    pub entries: Vec<String>,
    /// Output directory (defaults to "dist" if not provided)
    pub output_dir: Option<String>,
    /// Output format (esm, cjs, iife)
    pub format: Option<String>,
    /// Enable source maps
    pub sourcemap: Option<bool>,
    /// Working directory for resolution
    pub cwd: Option<String>,
}

/// Fob bundler for Node.js
#[napi]
pub struct Fob {
    config: BundleConfig,
    runtime: Arc<dyn Runtime>,
}

#[napi]
impl Fob {
    /// Create a new bundler instance
    #[napi(constructor)]
    pub fn new(config: BundleConfig) -> Result<Self> {
        let cwd = config
            .cwd
            .as_ref()
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| Error::from_reason("Failed to determine working directory"))?;

        let runtime: Arc<dyn Runtime> = Arc::new(
            NativeRuntime::new(cwd)
                .map_err(|e| Error::from_reason(format!("Failed to create runtime: {}", e)))?
        );

        Ok(Self { config, runtime })
    }

    /// Bundle the configured entries and return detailed bundle information
    #[napi]
    pub async fn bundle(&self) -> Result<BundleResult> {
        // Validation
        if self.config.entries.is_empty() {
            return Err(Error::from_reason("No entry points provided"));
        }

        // Parse format
        let format = match self.config.format.as_deref() {
            Some("esm") => OutputFormat::Esm,
            Some("cjs") => OutputFormat::Cjs,
            Some("iife") => OutputFormat::Iife,
            None => OutputFormat::Esm,
            Some(f) => return Err(Error::from_reason(format!("Unknown format: {}", f))),
        };

        let sourcemap = self.config.sourcemap.unwrap_or(false);
        let cwd = self.runtime.get_cwd()
            .map_err(|e| Error::from_reason(format!("Failed to get cwd: {}", e)))?;
        let out_dir = PathBuf::from(self.config.output_dir.as_deref().unwrap_or("dist"));

        // Create output directory
        fs::create_dir_all(&out_dir)
            .await
            .map_err(|e| Error::from_reason(format!("Failed to create output dir: {}", e)))?;

        // Build
        let build_result = if self.config.entries.len() == 1 {
            BuildOptions::library(self.config.entries[0].clone())
                .cwd(cwd)
                .format(format)
                .sourcemap(sourcemap)
                .runtime(self.runtime.clone())
                .build()
                .await
        } else {
            BuildOptions::components(self.config.entries.clone())
                .cwd(cwd)
                .format(format)
                .sourcemap(sourcemap)
                .runtime(self.runtime.clone())
                .build()
                .await
        }
        .map_err(|e| Error::from_reason(format!("Build failed: {}", e)))?;

        // Write files to disk using the built-in writer
        build_result
            .write_to_force(&out_dir)
            .map_err(|e| Error::from_reason(format!("Failed to write outputs: {}", e)))?;

        // Convert to NAPI result (this uses the From trait)
        Ok(BundleResult::from(build_result))
    }
}

/// Quick helper to bundle a single entry
#[napi]
pub async fn bundle_single(
    entry: String,
    output_dir: String,
    format: Option<String>,
) -> Result<BundleResult> {
    let config = BundleConfig {
        entries: vec![entry],
        output_dir: Some(output_dir),
        format,
        sourcemap: Some(true),
        cwd: None,
    };

    let bundler = Fob::new(config)?;
    bundler.bundle().await
}

/// Get the bundler version
#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

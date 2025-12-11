//! Fob bundler NAPI class

use crate::api::config::BundleConfig;
use crate::api::primitives::CodeSplittingConfig;
use crate::conversion::result::BundleResult;
use crate::core::bundler::CoreBundler;
use crate::error_mapper::map_bundler_error;
use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Common build options shared by preset functions.
#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct BuildOptions {
    /// Output directory (defaults to "dist")
    pub out_dir: Option<String>,
    /// Output format: "esm" (default), "cjs", or "iife"
    pub format: Option<String>,
    /// Source map generation: "true", "false", "inline", "hidden"
    pub sourcemap: Option<String>,
    /// Packages to externalize (not bundled)
    pub external: Option<Vec<String>>,
    /// Target platform: "browser" (default) or "node"
    pub platform: Option<String>,
    /// Enable minification
    pub minify: Option<bool>,
    /// Working directory for resolution
    pub cwd: Option<String>,
}

/// Options for app builds with code splitting.
#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct AppOptions {
    /// Output directory (defaults to "dist")
    pub out_dir: Option<String>,
    /// Output format: "esm" (default), "cjs", or "iife"
    pub format: Option<String>,
    /// Source map generation: "true", "false", "inline", "hidden"
    pub sourcemap: Option<String>,
    /// Packages to externalize
    pub external: Option<Vec<String>>,
    /// Target platform: "browser" (default) or "node"
    pub platform: Option<String>,
    /// Enable minification
    pub minify: Option<bool>,
    /// Working directory for resolution
    pub cwd: Option<String>,
    /// Code splitting configuration
    pub code_splitting: Option<CodeSplittingConfig>,
}

/// Fob bundler for Node.js
#[napi]
pub struct Fob {
    bundler: CoreBundler,
}

#[napi]
impl Fob {
    /// Create a new bundler instance with full configuration control.
    #[napi(constructor)]
    pub fn new(config: BundleConfig) -> Result<Self> {
        let bundler = CoreBundler::new(config).map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Self { bundler })
    }

    /// Bundle the configured entries and return detailed bundle information.
    #[napi]
    pub async fn bundle(&self) -> Result<BundleResult> {
        let result = self.bundler.bundle().await.map_err(|e| {
            let details = map_bundler_error(&e);
            Error::from_reason(details.to_napi_json_string())
        })?;

        Ok(result)
    }

    // === Preset Static Methods ===

    /// Build a standalone bundle (single entry, full bundling).
    ///
    /// Best for: Applications, scripts, or single-file outputs.
    ///
    /// ```typescript
    /// const result = await Fob.bundleEntry("src/index.ts", { outDir: "dist" });
    /// ```
    #[napi]
    pub async fn bundle_entry(
        entry: String,
        options: Option<BuildOptions>,
    ) -> Result<BundleResult> {
        let opts = options.unwrap_or_default();
        let config = BundleConfig {
            entries: vec![entry],
            output_dir: opts.out_dir,
            format: opts.format,
            sourcemap: opts.sourcemap,
            external: opts.external,
            platform: opts.platform,
            minify: opts.minify,
            cwd: opts.cwd,
            mdx: None,
            entry_mode: Some("shared".to_string()),
            code_splitting: None,
            external_from_manifest: None,
        };

        let fob = Self::new(config)?;
        fob.bundle().await
    }

    /// Build a library (single entry, externalize dependencies).
    ///
    /// Best for: npm packages, reusable libraries.
    /// Dependencies are marked as external and not bundled.
    ///
    /// ```typescript
    /// const result = await Fob.library("src/index.ts", {
    ///   outDir: "dist",
    ///   external: ["react", "react-dom"]
    /// });
    /// ```
    #[napi]
    pub async fn library(entry: String, options: Option<BuildOptions>) -> Result<BundleResult> {
        let opts = options.unwrap_or_default();
        let config = BundleConfig {
            entries: vec![entry],
            output_dir: opts.out_dir,
            format: opts.format,
            sourcemap: opts.sourcemap,
            external: opts.external,
            platform: opts.platform,
            minify: opts.minify,
            cwd: opts.cwd,
            mdx: None,
            entry_mode: Some("shared".to_string()),
            code_splitting: None,
            external_from_manifest: Some(true),
        };

        let fob = Self::new(config)?;
        fob.bundle().await
    }

    /// Build an app with code splitting (multiple entries, unified output).
    ///
    /// Best for: Web applications with multiple pages/routes.
    /// Shared dependencies are extracted into common chunks.
    ///
    /// ```typescript
    /// const result = await Fob.app(["src/client.tsx", "src/worker.ts"], {
    ///   outDir: "dist",
    ///   chunking: { minSize: 20000, minShareCount: 2 }
    /// });
    /// ```
    #[napi]
    pub async fn app(entries: Vec<String>, options: Option<AppOptions>) -> Result<BundleResult> {
        let opts = options.unwrap_or_default();
        let config = BundleConfig {
            entries,
            output_dir: opts.out_dir,
            format: opts.format,
            sourcemap: opts.sourcemap,
            external: opts.external,
            platform: opts.platform,
            minify: opts.minify,
            cwd: opts.cwd,
            mdx: None,
            entry_mode: Some("shared".to_string()),
            code_splitting: opts.code_splitting.clone(),
            external_from_manifest: None,
        };

        let fob = Self::new(config)?;
        fob.bundle().await
    }

    /// Build a component library (multiple entries, separate bundles).
    ///
    /// Best for: UI component libraries, design systems.
    /// Each entry produces an independent bundle with no shared chunks.
    ///
    /// ```typescript
    /// const result = await Fob.components(["src/Button.tsx", "src/Card.tsx"], {
    ///   outDir: "dist"
    /// });
    /// ```
    #[napi]
    pub async fn components(
        entries: Vec<String>,
        options: Option<BuildOptions>,
    ) -> Result<BundleResult> {
        let opts = options.unwrap_or_default();
        let config = BundleConfig {
            entries,
            output_dir: opts.out_dir,
            format: opts.format,
            sourcemap: opts.sourcemap,
            external: opts.external,
            platform: opts.platform,
            minify: opts.minify,
            cwd: opts.cwd,
            mdx: None,
            entry_mode: Some("isolated".to_string()),
            code_splitting: None,
            external_from_manifest: Some(true),
        };

        let fob = Self::new(config)?;
        fob.bundle().await
    }
}

//! Development mode builder that wraps the unified build function.
//!
//! Provides optimized rebuilds by:
//! - Keeping bundles in memory for instant serving
//! - Writing to disk asynchronously in the background
//! - Reusing build configuration and utilities

use crate::config::FobConfig;
use crate::dev::BundleCache;
use crate::error::Result;
use fob_bundler::builders::asset_processor;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing;

/// Development builder wrapping production build function.
///
/// Handles both initial builds (with disk write) and incremental rebuilds
/// (memory + async disk write).
pub struct DevBuilder {
    /// Base configuration for building
    config: FobConfig,
    /// Working directory
    cwd: std::path::PathBuf,
}

impl DevBuilder {
    /// Create a new development builder.
    ///
    /// # Arguments
    ///
    /// * `config` - Base bundler configuration
    /// * `cwd` - Working directory for resolving paths
    ///
    /// # Returns
    ///
    /// Configured DevBuilder instance
    pub fn new(config: FobConfig, cwd: std::path::PathBuf) -> Self {
        Self { config, cwd }
    }

    /// Perform initial build with disk write.
    ///
    /// This is called once on server startup to ensure output files exist.
    /// Delegates to the unified production build function.
    ///
    /// # Returns
    ///
    /// Tuple of (duration_ms, bundle_cache, asset_registry)
    ///
    /// # Errors
    ///
    /// Returns errors from the underlying build process
    pub async fn initial_build(
        &self,
    ) -> Result<(
        u64,
        BundleCache,
        Option<std::sync::Arc<fob_bundler::builders::asset_registry::AssetRegistry>>,
    )> {
        let start = Instant::now();

        let result = crate::commands::build::build_with_result(&self.config, &self.cwd).await?;
        let duration_ms = start.elapsed().as_millis() as u64;

        // Build cache by reading the output files, with URL rewriting if assets exist
        let cache = if let Some(ref registry) = result.asset_registry {
            self.build_cache_with_rewriting(registry).await?
        } else {
            self.build_cache_from_disk().await?
        };

        Ok((duration_ms, cache, result.asset_registry))
    }

    /// Perform incremental rebuild.
    ///
    /// This is optimized for dev mode:
    /// 1. Build in memory first (fast)
    /// 2. Populate cache for instant serving
    /// 3. Write to disk asynchronously
    ///
    /// # Returns
    ///
    /// Tuple of (duration_ms, bundle_cache, asset_registry)
    ///
    /// # Errors
    ///
    /// Returns errors from the build process
    pub async fn rebuild(
        &self,
    ) -> Result<(
        u64,
        BundleCache,
        Option<std::sync::Arc<fob_bundler::builders::asset_registry::AssetRegistry>>,
    )> {
        let start = Instant::now();

        // For now, delegate to unified build since we're using the production
        // build function which already writes to disk. In a full implementation,
        // we would intercept the bundle output before disk write.
        //
        // TODO: Enhance fob-core to support in-memory builds without disk I/O
        let result = crate::commands::build::build_with_result(&self.config, &self.cwd).await?;

        let duration_ms = start.elapsed().as_millis() as u64;

        // Build cache by reading the output files, with URL rewriting if assets exist
        let cache = if let Some(ref registry) = result.asset_registry {
            self.build_cache_with_rewriting(registry).await?
        } else {
            self.build_cache_from_disk().await?
        };

        Ok((duration_ms, cache, result.asset_registry))
    }

    /// Build cache from disk-written files.
    ///
    /// Reads the output directory and loads files into memory cache.
    /// This allows serving without repeated disk I/O.
    ///
    /// # Security
    ///
    /// - Only reads files from the configured output directory
    /// - Validates file paths to prevent directory traversal
    /// - Limits file size to prevent memory exhaustion
    pub async fn build_cache_from_disk(&self) -> Result<BundleCache> {
        use tokio::fs;

        let mut cache = BundleCache::new();
        let out_dir = if self.config.out_dir.is_absolute() {
            self.config.out_dir.clone()
        } else {
            self.cwd.join(&self.config.out_dir)
        };

        // Read directory entries
        let mut entries = fs::read_dir(&out_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Skip directories and non-files
            if !path.is_file() {
                continue;
            }

            // Security: Validate path is within output directory
            if !path.starts_with(&out_dir) {
                continue;
            }

            // Get file name for URL path
            let file_name = match path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };

            // Determine content type
            let content_type = Self::content_type_from_extension(&file_name);

            // Security: Limit file size (10MB max)
            const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;
            let metadata = entry.metadata().await?;
            if metadata.len() > MAX_FILE_SIZE {
                crate::ui::warning(&format!(
                    "Skipping large file {}: {} bytes",
                    file_name,
                    metadata.len()
                ));
                continue;
            }

            // Read file content
            let content = fs::read(&path).await?;

            // Add to cache with URL path
            let url_path = format!("/{}", file_name);
            cache.insert(url_path, content, content_type);
        }

        Ok(cache)
    }

    /// Determine MIME type from file extension.
    fn content_type_from_extension(filename: &str) -> String {
        if filename.ends_with(".js") || filename.ends_with(".mjs") {
            "application/javascript".to_string()
        } else if filename.ends_with(".map") {
            "application/json".to_string()
        } else if filename.ends_with(".d.ts") {
            "text/plain".to_string()
        } else if filename.ends_with(".css") {
            "text/css".to_string()
        } else if filename.ends_with(".html") {
            "text/html".to_string()
        } else if filename.ends_with(".wasm") {
            "application/wasm".to_string()
        } else {
            "application/octet-stream".to_string()
        }
    }

    /// Rewrite asset URLs in JavaScript files from disk.
    ///
    /// Transforms `new URL(path, import.meta.url)` patterns to direct URLs
    /// that point to the `/__fob_assets__/*` endpoint.
    ///
    /// This reads JavaScript files from the output directory, rewrites them,
    /// and returns them in the cache.
    ///
    /// # Arguments
    ///
    /// * `registry` - Asset registry with URL mappings
    ///
    /// # Returns
    ///
    /// Result containing rewritten cache
    async fn build_cache_with_rewriting(
        &self,
        registry: &Arc<fob_bundler::builders::asset_registry::AssetRegistry>,
    ) -> Result<BundleCache> {
        use tokio::fs;

        tracing::debug!("[URL_REWRITE] Building cache with URL rewriting");

        // Build URL map: specifier → public URL
        let mut url_map: HashMap<String, String> = HashMap::new();

        for asset in registry.all_assets() {
            if let Some(url_path) = &asset.url_path {
                tracing::debug!(
                    "[URL_REWRITE] Mapping: '{}' -> '{}'",
                    asset.specifier,
                    url_path
                );
                url_map.insert(asset.specifier.clone(), url_path.clone());
            }
        }

        tracing::debug!("[URL_REWRITE] URL map has {} entries", url_map.len());

        let mut cache = BundleCache::new();
        let out_dir = if self.config.out_dir.is_absolute() {
            self.config.out_dir.clone()
        } else {
            self.cwd.join(&self.config.out_dir)
        };

        // Read directory entries
        let mut entries = fs::read_dir(&out_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Skip directories and non-files
            if !path.is_file() {
                continue;
            }

            // Security: Validate path is within output directory
            if !path.starts_with(&out_dir) {
                continue;
            }

            // Get file name for URL path
            let file_name = match path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };

            // Determine content type
            let content_type = Self::content_type_from_extension(&file_name);

            // Security: Limit file size (10MB max)
            const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;
            let metadata = entry.metadata().await?;
            if metadata.len() > MAX_FILE_SIZE {
                crate::ui::warning(&format!(
                    "Skipping large file {}: {} bytes",
                    file_name,
                    metadata.len()
                ));
                continue;
            }

            // Read file content
            let content = fs::read(&path).await?;

            // Rewrite JavaScript files if we have assets to rewrite
            let final_content = if content_type == "application/javascript" && !url_map.is_empty() {
                tracing::debug!("[URL_REWRITE] Processing JS file: {}", file_name);
                if let Ok(code) = String::from_utf8(content.clone()) {
                    let rewritten = asset_processor::rewrite_urls(&code, &url_map);
                    if rewritten != code {
                        tracing::debug!("[URL_REWRITE] URLs were rewritten in {}", file_name);
                    } else {
                        tracing::debug!("[URL_REWRITE] No changes needed for {}", file_name);
                    }
                    rewritten.into_bytes()
                } else {
                    tracing::warn!("[URL_REWRITE] Failed to parse {} as UTF-8", file_name);
                    content
                }
            } else {
                content
            };

            // Add to cache with URL path
            let url_path = format!("/{}", file_name);
            cache.insert(url_path, final_content, content_type);
        }

        Ok(cache)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // DevBuilder tests removed - mode detection no longer exists
    // Content type helpers still tested below

    #[test]
    fn test_content_type_js() {
        assert_eq!(
            DevBuilder::content_type_from_extension("bundle.js"),
            "application/javascript"
        );
        assert_eq!(
            DevBuilder::content_type_from_extension("module.mjs"),
            "application/javascript"
        );
    }

    #[test]
    fn test_content_type_map() {
        assert_eq!(
            DevBuilder::content_type_from_extension("bundle.js.map"),
            "application/json"
        );
    }

    #[test]
    fn test_content_type_dts() {
        assert_eq!(
            DevBuilder::content_type_from_extension("types.d.ts"),
            "text/plain"
        );
    }

    #[test]
    fn test_content_type_unknown() {
        assert_eq!(
            DevBuilder::content_type_from_extension("file.xyz"),
            "application/octet-stream"
        );
    }
}

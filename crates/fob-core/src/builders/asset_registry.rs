//! Asset registry for tracking static assets discovered during bundling.
//!
//! This module provides a thread-safe registry that tracks assets referenced
//! via `new URL(path, import.meta.url)` patterns in JavaScript/TypeScript code.

use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Thread-safe asset registry.
///
/// Tracks all assets discovered during the bundling process, including their
/// resolved paths, metadata, and URL mappings.
#[derive(Debug, Clone, Default)]
pub struct AssetRegistry {
    inner: Arc<RwLock<AssetRegistryInner>>,
}

#[derive(Debug, Default)]
struct AssetRegistryInner {
    /// Map of resolved absolute path → asset info
    assets: FxHashMap<PathBuf, AssetInfo>,

    /// Reverse lookup: URL path → resolved absolute path
    /// Used by dev server to quickly find assets by URL
    url_to_path: FxHashMap<String, PathBuf>,
}

/// Information about a discovered asset.
#[derive(Debug, Clone)]
pub struct AssetInfo {
    /// Absolute path to the asset file
    pub source_path: PathBuf,

    /// Module that referenced this asset
    pub referrer: String,

    /// Original specifier from the source code
    /// Example: "../wasm/web/file.wasm"
    pub specifier: String,

    /// Content type (MIME type)
    pub content_type: String,

    /// File size in bytes (if known)
    pub size: Option<u64>,

    /// URL path for serving (e.g., "/__fob_assets__/hash.wasm")
    pub url_path: Option<String>,

    /// Hash of content (for production builds)
    pub content_hash: Option<String>,
}

impl AssetRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a discovered asset.
    ///
    /// # Arguments
    ///
    /// * `source_path` - Absolute path to the asset file
    /// * `referrer` - Module ID that referenced this asset
    /// * `specifier` - Original import specifier
    ///
    /// # Returns
    ///
    /// The asset info, either newly created or existing
    pub fn register(
        &self,
        source_path: PathBuf,
        referrer: String,
        specifier: String,
    ) -> AssetInfo {
        let mut inner = self.inner.write();

        // Check if already registered
        if let Some(existing) = inner.assets.get(&source_path) {
            return existing.clone();
        }

        // Determine content type from extension
        let content_type = Self::content_type_from_path(&source_path);

        // Try to get file size
        let size = std::fs::metadata(&source_path)
            .ok()
            .map(|m| m.len());

        let info = AssetInfo {
            source_path: source_path.clone(),
            referrer,
            specifier,
            content_type,
            size,
            url_path: None,
            content_hash: None,
        };

        inner.assets.insert(source_path, info.clone());
        info
    }

    /// Set the URL path for an asset (used in dev mode).
    pub fn set_url_path(&self, source_path: &Path, url_path: String) {
        let mut inner = self.inner.write();

        if let Some(info) = inner.assets.get_mut(source_path) {
            info.url_path = Some(url_path.clone());
            inner.url_to_path.insert(url_path, source_path.to_path_buf());
        }
    }

    /// Set the content hash for an asset (used in production mode).
    pub fn set_content_hash(&self, source_path: &Path, hash: String) {
        let mut inner = self.inner.write();

        if let Some(info) = inner.assets.get_mut(source_path) {
            info.content_hash = Some(hash);
        }
    }

    /// Get asset info by source path.
    pub fn get(&self, source_path: &Path) -> Option<AssetInfo> {
        let inner = self.inner.read();
        inner.assets.get(source_path).cloned()
    }

    /// Get asset info by URL path (for dev server lookups).
    pub fn get_by_url(&self, url_path: &str) -> Option<AssetInfo> {
        let inner = self.inner.read();
        let source_path = inner.url_to_path.get(url_path)?;
        inner.assets.get(source_path).cloned()
    }

    /// Get all registered assets.
    pub fn all_assets(&self) -> Vec<AssetInfo> {
        let inner = self.inner.read();
        inner.assets.values().cloned().collect()
    }

    /// Get number of registered assets.
    pub fn len(&self) -> usize {
        let inner = self.inner.read();
        inner.assets.len()
    }

    /// Check if registry is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all assets from the registry.
    pub fn clear(&self) {
        let mut inner = self.inner.write();
        inner.assets.clear();
        inner.url_to_path.clear();
    }

    /// Determine content type from file path.
    fn content_type_from_path(path: &Path) -> String {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "wasm" => "application/wasm",
            "js" | "mjs" => "application/javascript",
            "json" => "application/json",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "svg" => "image/svg+xml",
            "webp" => "image/webp",
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            "ttf" => "font/ttf",
            "css" => "text/css",
            "html" => "text/html",
            "txt" => "text/plain",
            _ => "application/octet-stream",
        }.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_asset() {
        let registry = AssetRegistry::new();
        let path = PathBuf::from("/test/file.wasm");

        let info = registry.register(
            path.clone(),
            "module.js".to_string(),
            "./file.wasm".to_string(),
        );

        assert_eq!(info.source_path, path);
        assert_eq!(info.referrer, "module.js");
        assert_eq!(info.specifier, "./file.wasm");
        assert_eq!(info.content_type, "application/wasm");
    }

    #[test]
    fn test_url_mapping() {
        let registry = AssetRegistry::new();
        let path = PathBuf::from("/test/file.wasm");

        registry.register(
            path.clone(),
            "module.js".to_string(),
            "./file.wasm".to_string(),
        );

        registry.set_url_path(&path, "/__fob_assets__/abc123.wasm".to_string());

        let info = registry.get_by_url("/__fob_assets__/abc123.wasm");
        assert!(info.is_some());
        assert_eq!(info.unwrap().source_path, path);
    }

    #[test]
    fn test_content_type_detection() {
        assert_eq!(
            AssetRegistry::content_type_from_path(Path::new("file.wasm")),
            "application/wasm"
        );
        assert_eq!(
            AssetRegistry::content_type_from_path(Path::new("file.js")),
            "application/javascript"
        );
        assert_eq!(
            AssetRegistry::content_type_from_path(Path::new("file.png")),
            "image/png"
        );
    }
}

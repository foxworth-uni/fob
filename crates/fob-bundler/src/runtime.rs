//! Bundler-specific Runtime implementation
//!
//! This module provides `BundlerRuntime`, a Runtime implementation that combines
//! virtual file support with filesystem access. Virtual files are checked first,
//! then the runtime falls back to the underlying filesystem.

use async_trait::async_trait;
use fob_graph::runtime::{FileMetadata, Runtime, RuntimeError, RuntimeResult};
use parking_lot::RwLock;
use path_clean::PathClean;
use rustc_hash::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Runtime implementation that combines virtual files with filesystem access
///
/// This runtime checks virtual files first (in-memory), then falls back to
/// the filesystem. This allows plugins to work seamlessly with both virtual
/// and real files.
#[derive(Debug)]
pub struct BundlerRuntime {
    /// Virtual files stored in memory
    virtual_files: Arc<RwLock<FxHashMap<PathBuf, Vec<u8>>>>,
    /// Current working directory for resolving relative paths
    cwd: PathBuf,
}

impl BundlerRuntime {
    /// Create a new BundlerRuntime with the given working directory
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        Self {
            virtual_files: Arc::new(RwLock::new(FxHashMap::default())),
            cwd: cwd.into(),
        }
    }

    /// Add a virtual file to the runtime
    ///
    /// The path is normalized before storage to ensure consistent lookup.
    pub fn add_virtual_file(&self, path: impl Into<PathBuf>, content: impl Into<Vec<u8>>) {
        let path_buf: PathBuf = path.into();
        let normalized = self.normalize_for_lookup(&path_buf);
        self.virtual_files
            .write()
            .insert(normalized, content.into());
    }

    /// Check if a path exists as a virtual file
    ///
    /// Normalizes the path before checking to ensure consistent lookup.
    pub fn has_virtual_file(&self, path: &Path) -> bool {
        let normalized = self.normalize_for_lookup(path);
        self.virtual_files.read().contains_key(&normalized)
    }

    /// Resolve a path relative to the current working directory
    fn resolve_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.cwd.join(path)
        }
    }

    /// Normalize a path for virtual file lookup
    ///
    /// This ensures that paths like "/foo/bar.js" and "./bar.js" (when cwd is /foo)
    /// are treated consistently when looking up virtual files.
    fn normalize_for_lookup(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            // For relative paths, resolve against cwd and normalize
            let resolved = self.cwd.join(path);
            // Normalize by cleaning redundant components
            resolved.clean()
        }
    }
}

#[async_trait]
impl Runtime for BundlerRuntime {
    async fn read_file(&self, path: &Path) -> RuntimeResult<Vec<u8>> {
        // Check virtual files first (normalize path for consistent lookup)
        let normalized = self.normalize_for_lookup(path);
        if let Some(content) = self.virtual_files.read().get(&normalized) {
            return Ok(content.clone());
        }

        // Fall back to filesystem
        let full_path = self.resolve_path(path);

        #[cfg(not(target_family = "wasm"))]
        {
            use tokio::task;
            let path = full_path.clone();
            task::spawn_blocking(move || {
                std::fs::read(&path).map_err(|e| {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        RuntimeError::FileNotFound(path.clone())
                    } else {
                        RuntimeError::Io(format!("Failed to read {}: {}", path.display(), e))
                    }
                })
            })
            .await
            .map_err(|e| RuntimeError::Other(format!("Task join error: {}", e)))?
        }

        #[cfg(target_family = "wasm")]
        {
            // On WASM, we'd need to delegate to a provided runtime
            // For now, return an error indicating this isn't supported
            return Err(RuntimeError::Other(
                "BundlerRuntime filesystem access not available on WASM. \
                Provide a Runtime implementation via BuildOptions::runtime()"
                    .to_string(),
            ));
        }
    }

    async fn write_file(&self, path: &Path, content: &[u8]) -> RuntimeResult<()> {
        let full_path = self.resolve_path(path);

        #[cfg(not(target_family = "wasm"))]
        {
            use tokio::task;
            let path = full_path.clone();
            let content = content.to_vec();
            task::spawn_blocking(move || {
                std::fs::write(&path, content).map_err(|e| {
                    RuntimeError::Io(format!("Failed to write {}: {}", path.display(), e))
                })
            })
            .await
            .map_err(|e| RuntimeError::Other(format!("Task join error: {}", e)))?
        }

        #[cfg(target_family = "wasm")]
        {
            return Err(RuntimeError::Other(
                "BundlerRuntime filesystem write not available on WASM".to_string(),
            ));
        }
    }

    async fn metadata(&self, path: &Path) -> RuntimeResult<FileMetadata> {
        // Check virtual files first (normalize path for consistent lookup)
        let normalized = self.normalize_for_lookup(path);
        if let Some(content) = self.virtual_files.read().get(&normalized) {
            return Ok(FileMetadata {
                size: content.len() as u64,
                is_dir: false,
                is_file: true,
                modified: None,
            });
        }

        // Fall back to filesystem
        let full_path = self.resolve_path(path);

        #[cfg(not(target_family = "wasm"))]
        {
            use tokio::task;
            let path = full_path.clone();
            task::spawn_blocking(move || {
                let metadata = std::fs::metadata(&path).map_err(|e| {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        RuntimeError::FileNotFound(path.clone())
                    } else {
                        RuntimeError::Io(format!(
                            "Failed to get metadata for {}: {}",
                            path.display(),
                            e
                        ))
                    }
                })?;

                let modified = metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as u64);

                Ok(FileMetadata {
                    size: metadata.len(),
                    is_dir: metadata.is_dir(),
                    is_file: metadata.is_file(),
                    modified,
                })
            })
            .await
            .map_err(|e| RuntimeError::Other(format!("Task join error: {}", e)))?
        }

        #[cfg(target_family = "wasm")]
        {
            return Err(RuntimeError::Other(
                "BundlerRuntime metadata not available on WASM".to_string(),
            ));
        }
    }

    fn exists(&self, path: &Path) -> bool {
        // Check virtual files first (normalize path for consistent lookup)
        let normalized = self.normalize_for_lookup(path);
        if self.virtual_files.read().contains_key(&normalized) {
            return true;
        }

        // Fall back to filesystem
        let full_path = self.resolve_path(path);
        full_path.exists()
    }

    fn resolve(&self, specifier: &str, from: &Path) -> RuntimeResult<PathBuf> {
        // Handle absolute paths
        if Path::new(specifier).is_absolute() {
            return Ok(PathBuf::from(specifier));
        }

        // Handle relative paths
        if specifier.starts_with("./") || specifier.starts_with("../") {
            let from_dir = from.parent().unwrap_or(Path::new(""));
            let resolved = self.resolve_path(&from_dir.join(specifier));

            // Try to canonicalize to resolve symlinks and normalize paths
            // If canonicalization fails (e.g., path doesn't exist yet), that's ok -
            // we return the resolved path anyway. The caller can check existence separately.
            #[cfg(not(target_family = "wasm"))]
            {
                if let Ok(canonical) = resolved.canonicalize() {
                    return Ok(canonical);
                }
            }

            return Ok(resolved);
        }

        // For bare specifiers, return as-is
        // The bundler's node resolution will handle these
        Ok(PathBuf::from(specifier))
    }

    async fn create_dir(&self, path: &Path, recursive: bool) -> RuntimeResult<()> {
        let full_path = self.resolve_path(path);

        #[cfg(not(target_family = "wasm"))]
        {
            use tokio::task;
            let path = full_path.clone();
            task::spawn_blocking(move || {
                let result = if recursive {
                    std::fs::create_dir_all(&path)
                } else {
                    std::fs::create_dir(&path)
                };

                result.map_err(|e| {
                    RuntimeError::Io(format!(
                        "Failed to create directory {}: {}",
                        path.display(),
                        e
                    ))
                })
            })
            .await
            .map_err(|e| RuntimeError::Other(format!("Task join error: {}", e)))?
        }

        #[cfg(target_family = "wasm")]
        {
            return Err(RuntimeError::Other(
                "BundlerRuntime create_dir not available on WASM".to_string(),
            ));
        }
    }

    async fn remove_file(&self, path: &Path) -> RuntimeResult<()> {
        let full_path = self.resolve_path(path);

        #[cfg(not(target_family = "wasm"))]
        {
            use tokio::task;
            let path = full_path.clone();
            task::spawn_blocking(move || {
                std::fs::remove_file(&path).map_err(|e| {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        RuntimeError::FileNotFound(path.clone())
                    } else {
                        RuntimeError::Io(format!("Failed to remove {}: {}", path.display(), e))
                    }
                })
            })
            .await
            .map_err(|e| RuntimeError::Other(format!("Task join error: {}", e)))?
        }

        #[cfg(target_family = "wasm")]
        {
            return Err(RuntimeError::Other(
                "BundlerRuntime remove_file not available on WASM".to_string(),
            ));
        }
    }

    async fn read_dir(&self, path: &Path) -> RuntimeResult<Vec<String>> {
        let full_path = self.resolve_path(path);

        #[cfg(not(target_family = "wasm"))]
        {
            use tokio::task;
            let path = full_path.clone();
            task::spawn_blocking(move || {
                let entries = std::fs::read_dir(&path).map_err(|e| {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        RuntimeError::FileNotFound(path.clone())
                    } else {
                        RuntimeError::Io(format!(
                            "Failed to read directory {}: {}",
                            path.display(),
                            e
                        ))
                    }
                })?;

                let mut result = Vec::new();
                for entry in entries {
                    let entry = entry.map_err(|e| {
                        RuntimeError::Io(format!("Failed to read directory entry: {}", e))
                    })?;

                    if let Some(name) = entry.file_name().to_str() {
                        result.push(name.to_string());
                    }
                }

                Ok(result)
            })
            .await
            .map_err(|e| RuntimeError::Other(format!("Task join error: {}", e)))?
        }

        #[cfg(target_family = "wasm")]
        {
            return Err(RuntimeError::Other(
                "BundlerRuntime read_dir not available on WASM".to_string(),
            ));
        }
    }

    fn get_cwd(&self) -> RuntimeResult<PathBuf> {
        Ok(self.cwd.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_virtual_file() {
        let runtime = BundlerRuntime::new(".");
        runtime.add_virtual_file("virtual:test.js", b"export const x = 1;");

        let content = runtime
            .read_file(Path::new("virtual:test.js"))
            .await
            .unwrap();
        assert_eq!(content, b"export const x = 1;");
    }

    #[tokio::test]
    async fn test_virtual_file_exists() {
        let runtime = BundlerRuntime::new(".");
        runtime.add_virtual_file("virtual:test.js", b"content");

        assert!(runtime.exists(Path::new("virtual:test.js")));
        assert!(!runtime.exists(Path::new("virtual:nonexistent.js")));
    }

    #[tokio::test]
    async fn test_filesystem_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, b"filesystem content").unwrap();

        let runtime = BundlerRuntime::new(temp_dir.path());

        let content = runtime.read_file(&file_path).await.unwrap();
        assert_eq!(content, b"filesystem content");
    }

    #[tokio::test]
    async fn test_virtual_takes_precedence() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, b"filesystem content").unwrap();

        let runtime = BundlerRuntime::new(temp_dir.path());
        runtime.add_virtual_file(&file_path, b"virtual content");

        // Virtual file should take precedence
        let content = runtime.read_file(&file_path).await.unwrap();
        assert_eq!(content, b"virtual content");
    }
}

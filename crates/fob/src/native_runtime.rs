//! Native Runtime Implementation
//!
//! This module provides a `Runtime` trait implementation for native (non-WASM)
//! environments where standard filesystem operations are available.
//!
//! ## Why This Exists
//!
//! While `std::fs` works perfectly on native platforms, we wrap it in the
//! Runtime trait to provide a consistent interface across all platforms.
//! This allows the bundler core to be platform-agnostic.
//!
//! ## Architecture
//!
//! ```text
//! Rust (Native)
//! ┌─────────────────┐
//! │ NativeRuntime   │
//! │  .read_file()   │────▶ std::fs::read()
//! │  .write_file()  │────▶ std::fs::write()
//! │  .exists()      │────▶ std::path::Path::exists()
//! └─────────────────┘
//!          │
//!          ▼
//!   ┌──────────────┐
//!   │ OS Filesystem│
//!   └──────────────┘
//! ```

// NativeRuntime is platform-specific and wraps std::fs by design
#![allow(clippy::disallowed_methods)]

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::task;

use crate::runtime::{FileMetadata, Runtime, RuntimeError, RuntimeResult};

/// Native filesystem Runtime implementation using `std::fs`.
///
/// This implementation provides async wrappers around synchronous `std::fs`
/// operations using tokio's spawn_blocking to avoid blocking the async runtime.
///
/// # Educational Note: Async File I/O
///
/// Standard library file operations are blocking (they wait for the OS).
/// To use them in async code without blocking the executor, we run them in
/// a separate thread pool using `tokio::task::spawn_blocking`.
///
/// # Example
///
/// ```rust,ignore
/// use fob::native_runtime::NativeRuntime;
/// use fob::runtime::Runtime;
///
/// let runtime = NativeRuntime;
/// let content = runtime.read_file(Path::new("file.txt")).await?;
/// ```
#[derive(Debug, Clone, Copy)]
pub struct NativeRuntime;

impl NativeRuntime {
    /// Create a new NativeRuntime instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for NativeRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for NativeRuntime {
    /// Read a file from the native filesystem.
    ///
    /// # Educational Note: spawn_blocking
    ///
    /// `std::fs::read` is a blocking operation that waits for disk I/O.
    /// We use `spawn_blocking` to run it in a dedicated thread pool,
    /// preventing it from blocking async tasks.
    async fn read_file(&self, path: &Path) -> RuntimeResult<Vec<u8>> {
        let path = path.to_path_buf();

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

    /// Write a file to the native filesystem.
    async fn write_file(&self, path: &Path, content: &[u8]) -> RuntimeResult<()> {
        let path = path.to_path_buf();
        let content = content.to_vec();

        task::spawn_blocking(move || {
            std::fs::write(&path, content)
                .map_err(|e| RuntimeError::Io(format!("Failed to write {}: {}", path.display(), e)))
        })
        .await
        .map_err(|e| RuntimeError::Other(format!("Task join error: {}", e)))?
    }

    /// Get file metadata from the native filesystem.
    async fn metadata(&self, path: &Path) -> RuntimeResult<FileMetadata> {
        let path = path.to_path_buf();

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

            // Get modification time (platform-specific)
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

    /// Check if a path exists on the native filesystem.
    ///
    /// # Educational Note: Synchronous Methods
    ///
    /// This method is synchronous in the trait (no async), so we can call
    /// `std::path::Path::exists()` directly without spawn_blocking.
    /// It's a quick metadata check that won't block significantly.
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    /// Resolve a module specifier on the native filesystem.
    ///
    /// # Educational Note: Path Resolution
    ///
    /// This handles relative and absolute path resolution.
    /// For bare specifiers (like "lodash"), the bundler's node resolution
    /// logic will handle them separately.
    fn resolve(&self, specifier: &str, from: &Path) -> RuntimeResult<PathBuf> {
        // Handle absolute paths
        if Path::new(specifier).is_absolute() {
            return Ok(PathBuf::from(specifier));
        }

        // Handle relative paths
        if specifier.starts_with("./") || specifier.starts_with("../") {
            let from_dir = from.parent().unwrap_or(Path::new(""));
            let resolved = from_dir.join(specifier);

            // Canonicalize to resolve .. and symlinks
            return resolved
                .canonicalize()
                .map_err(|e| RuntimeError::ResolutionFailed {
                    specifier: specifier.to_string(),
                    from: from.to_path_buf(),
                    reason: format!("Canonicalization failed: {}", e),
                });
        }

        // For bare specifiers, return as-is
        // The bundler's node resolution will handle these
        Ok(PathBuf::from(specifier))
    }

    /// Create a directory on the native filesystem.
    async fn create_dir(&self, path: &Path, recursive: bool) -> RuntimeResult<()> {
        let path = path.to_path_buf();

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

    /// Remove a file from the native filesystem.
    async fn remove_file(&self, path: &Path) -> RuntimeResult<()> {
        let path = path.to_path_buf();

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

    /// Read directory contents from the native filesystem.
    async fn read_dir(&self, path: &Path) -> RuntimeResult<Vec<String>> {
        let path = path.to_path_buf();

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

    /// Get the current working directory from the operating system.
    ///
    /// # Educational Note: OS Working Directory
    ///
    /// Delegates to `std::env::current_dir()` to get the actual OS-level
    /// current working directory. This method abstracts the OS call behind
    /// the Runtime trait, enabling platform-agnostic code.
    fn get_cwd(&self) -> RuntimeResult<PathBuf> {
        std::env::current_dir().map_err(|e| {
            RuntimeError::Io(format!("Failed to get current working directory: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_read_write_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let runtime = NativeRuntime::new();

        // Write file
        let content = b"Hello, World!";
        runtime.write_file(&file_path, content).await.unwrap();

        // Read file
        let read_content = runtime.read_file(&file_path).await.unwrap();
        assert_eq!(read_content, content);
    }

    #[tokio::test]
    async fn test_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, b"test content").unwrap();

        let runtime = NativeRuntime::new();
        let metadata = runtime.metadata(&file_path).await.unwrap();

        assert!(metadata.is_file);
        assert!(!metadata.is_dir);
        assert_eq!(metadata.size, 12); // "test content" is 12 bytes
    }

    #[tokio::test]
    async fn test_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let runtime = NativeRuntime::new();

        // Should not exist initially
        assert!(!runtime.exists(&file_path));

        // Create file
        fs::write(&file_path, b"test").unwrap();

        // Should exist now
        assert!(runtime.exists(&file_path));
    }

    #[tokio::test]
    async fn test_read_dir() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("file1.txt"), b"test1").unwrap();
        fs::write(temp_dir.path().join("file2.txt"), b"test2").unwrap();

        let runtime = NativeRuntime::new();
        let mut entries = runtime.read_dir(temp_dir.path()).await.unwrap();
        entries.sort();

        assert_eq!(entries, vec!["file1.txt", "file2.txt"]);
    }

    #[tokio::test]
    async fn test_create_dir() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("subdir");

        let runtime = NativeRuntime::new();
        runtime.create_dir(&dir_path, false).await.unwrap();

        assert!(dir_path.exists());
        assert!(dir_path.is_dir());
    }

    #[tokio::test]
    async fn test_create_dir_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("a").join("b").join("c");

        let runtime = NativeRuntime::new();
        runtime.create_dir(&nested_path, true).await.unwrap();

        assert!(nested_path.exists());
        assert!(nested_path.is_dir());
    }
}

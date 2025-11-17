//! WASM Runtime Implementation
//!
//! This module provides a `Runtime` trait implementation for WASM environments.
//! For WASI targets (wasm32-wasip1, wasm32-wasip1-threads), this uses the WASI
//! filesystem APIs. For other WASM targets, this provides a minimal implementation
//! that can be extended with platform-specific filesystem bridges.
//!
//! ## Why This Exists
//!
//! WASM targets need a Runtime implementation that doesn't rely on tokio::fs
//! or std::fs directly (which may not be available on all WASM targets).
//! This abstraction allows the bundler core to work across different WASM environments.
//!
//! ## Architecture
//!
//! ```text
//! Rust (WASM)
//! ┌─────────────────┐
//! │ WasmRuntime     │
//! │  .read_file()   │────▶ std::fs::read() (WASI) or platform bridge
//! │  .write_file()  │────▶ std::fs::write() (WASI) or platform bridge
//! │  .exists()      │────▶ std::path::Path::exists() (WASI)
//! └─────────────────┘
//!          │
//!          ▼
//!   ┌──────────────┐
//!   │ WASI FS APIs │
//!   └──────────────┘
//! ```

use async_trait::async_trait;
use std::path::{Path, PathBuf};

use crate::runtime::{FileMetadata, Runtime, RuntimeError, RuntimeResult};

/// WASM filesystem Runtime implementation.
///
/// This implementation provides async wrappers around synchronous filesystem
/// operations. For WASI targets, it uses std::fs directly (WASI provides
/// filesystem support). For other WASM targets, it can be extended with
/// platform-specific bridges.
///
/// # Educational Note: WASM Filesystem Access
///
/// WASI (WebAssembly System Interface) provides filesystem access for WASM
/// modules. Unlike browser WASM, WASI targets can access a real or virtual
/// filesystem. This runtime wraps those operations in async for consistency
/// with the Runtime trait.
///
/// # Example
///
/// ```rust,ignore
/// use fob_core::wasm_runtime::WasmRuntime;
/// use fob_core::runtime::Runtime;
///
/// let runtime = WasmRuntime::new();
/// let content = runtime.read_file(Path::new("file.txt")).await?;
/// ```
#[derive(Debug, Clone, Copy)]
pub struct WasmRuntime;

impl WasmRuntime {
    /// Create a new WasmRuntime instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for WasmRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_family = "wasm")]
#[async_trait(?Send)]
impl Runtime for WasmRuntime {
    /// Read a file from the WASM filesystem.
    ///
    /// # Educational Note: WASI File I/O
    ///
    /// On WASI targets, std::fs::read is available and works synchronously.
    /// We wrap it in an async block to match the async trait signature.
    /// For WASI, std::fs operations are available and execute synchronously.
    async fn read_file(&self, path: &Path) -> RuntimeResult<Vec<u8>> {
        let path = path.to_path_buf();
        
        // WASI provides std::fs, so we can use it directly
        // Wrap in async block for trait compatibility
        async move {
            std::fs::read(&path).map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    RuntimeError::FileNotFound(path.clone())
                } else {
                    RuntimeError::Io(format!("Failed to read {}: {}", path.display(), e))
                }
            })
        }
        .await
    }

    /// Write a file to the WASM filesystem.
    async fn write_file(&self, path: &Path, content: &[u8]) -> RuntimeResult<()> {
        let path = path.to_path_buf();
        let content = content.to_vec();

        async move {
            std::fs::write(&path, content).map_err(|e| {
                RuntimeError::Io(format!("Failed to write {}: {}", path.display(), e))
            })
        }
        .await
    }

    /// Get file metadata from the WASM filesystem.
    async fn metadata(&self, path: &Path) -> RuntimeResult<FileMetadata> {
        let path = path.to_path_buf();

        async move {
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
        }
        .await
    }

    /// Check if a path exists on the WASM filesystem.
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    /// Resolve a module specifier on the WASM filesystem.
    fn resolve(&self, specifier: &str, from: &Path) -> RuntimeResult<PathBuf> {
        // Handle absolute paths
        if Path::new(specifier).is_absolute() {
            return Ok(PathBuf::from(specifier));
        }

        // Handle relative paths
        if specifier.starts_with("./") || specifier.starts_with("../") {
            let from_dir = from.parent().unwrap_or(Path::new(""));
            let resolved = from_dir.join(specifier);

            // On WASI, canonicalize may work, but we'll handle errors gracefully
            resolved.canonicalize().map_err(|e| {
                RuntimeError::ResolutionFailed {
                    specifier: specifier.to_string(),
                    from: from.to_path_buf(),
                    reason: format!("Canonicalization failed: {}", e),
                }
            })
        } else {
            // For bare specifiers, return as-is
            Ok(PathBuf::from(specifier))
        }
    }

    /// Create a directory on the WASM filesystem.
    async fn create_dir(&self, path: &Path, recursive: bool) -> RuntimeResult<()> {
        let path = path.to_path_buf();

        async move {
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
        }
        .await
    }

    /// Remove a file from the WASM filesystem.
    async fn remove_file(&self, path: &Path) -> RuntimeResult<()> {
        let path = path.to_path_buf();

        async move {
            std::fs::remove_file(&path).map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    RuntimeError::FileNotFound(path.clone())
                } else {
                    RuntimeError::Io(format!("Failed to remove {}: {}", path.display(), e))
                }
            })
        }
        .await
    }

    /// Read directory contents from the WASM filesystem.
    async fn read_dir(&self, path: &Path) -> RuntimeResult<Vec<String>> {
        let path = path.to_path_buf();

        async move {
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
        }
        .await
    }

    /// Get the current working directory.
    ///
    /// # Educational Note: Virtual Working Directory
    ///
    /// On WASI, std::env::current_dir() may work if the WASM runtime
    /// provides a working directory. Otherwise, this returns a virtual
    /// root directory.
    fn get_cwd(&self) -> RuntimeResult<PathBuf> {
        // Try to get actual cwd on WASI, fallback to virtual root
        std::env::current_dir()
            .or_else(|_| Ok::<PathBuf, std::io::Error>(PathBuf::from("/")))
            .map_err(|e| {
                RuntimeError::Io(format!("Failed to get current working directory: {}", e))
            })
    }
}


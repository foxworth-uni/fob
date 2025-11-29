//! Platform runtime abstraction for Fob bundler
//!
//! This module defines the `Runtime` trait that abstracts platform-specific
//! operations like file I/O and module resolution. Platform bindings
//! (joy-native, joy-wasm) implement this trait to provide platform-specific behavior.

// Platform-specific runtime implementations
#[cfg(not(target_family = "wasm"))]
pub mod native;

#[cfg(target_family = "wasm")]
pub mod wasm;

// Test utilities (available in test builds)
#[cfg(any(
    all(any(test, doctest), not(target_family = "wasm")),
    all(feature = "test-utils", not(target_family = "wasm"))
))]
pub mod test_utils;

use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Result type for runtime operations
pub type RuntimeResult<T> = Result<T, RuntimeError>;

/// Errors that can occur during runtime operations
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(String),

    /// Module resolution failed
    #[error("Failed to resolve module '{specifier}' from '{from}': {reason}")]
    ResolutionFailed {
        specifier: String,
        from: PathBuf,
        reason: String,
    },

    /// Other runtime error
    #[error("Runtime error: {0}")]
    Other(String),
}

/// File metadata
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// File size in bytes
    pub size: u64,
    /// Whether this is a directory
    pub is_dir: bool,
    /// Whether this is a file
    pub is_file: bool,
    /// Last modified timestamp (milliseconds since epoch)
    pub modified: Option<u64>,
}

/// Platform runtime trait
///
/// This trait abstracts platform-specific operations. Platform bindings
/// implement this trait to provide file I/O, module resolution, and other
/// platform-dependent functionality.
///
/// # Example
///
/// ```rust,ignore
/// use crate::runtime::{Runtime, RuntimeResult};
/// use async_trait::async_trait;
///
/// struct MyRuntime;
///
/// #[async_trait]
/// impl Runtime for MyRuntime {
///     async fn read_file(&self, path: &Path) -> RuntimeResult<Vec<u8>> {
///         // Platform-specific implementation
///         std::fs::read(path).map_err(|e| RuntimeError::Io(e.to_string()))
///     }
///
///     // ... implement other methods
/// }
/// ```
// WASM target: Single-threaded execution
// Educational Note: WASM is always single-threaded, so Send/Sync don't affect
// runtime behavior. However, we still declare the trait with Send + Sync to
// satisfy trait object requirements (Arc<dyn Runtime>). The Send bound on
// futures is conditionally removed using ?Send from async_trait.
//
// IMPORTANT: We keep Send + Sync on the trait itself because:
// 1. Arc<dyn Runtime> requires Send + Sync
// 2. These are just marker traits on WASM (no actual threading)
// 3. The ?Send applies to the futures returned by methods, not the trait
#[cfg(target_family = "wasm")]
#[async_trait(?Send)]
pub trait Runtime: Send + Sync + std::fmt::Debug {
    /// Read a file from the filesystem
    async fn read_file(&self, path: &Path) -> RuntimeResult<Vec<u8>>;

    /// Write a file to the filesystem
    async fn write_file(&self, path: &Path, content: &[u8]) -> RuntimeResult<()>;

    /// Get file metadata
    async fn metadata(&self, path: &Path) -> RuntimeResult<FileMetadata>;

    /// Check if a path exists
    fn exists(&self, path: &Path) -> bool;

    /// Resolve a module specifier
    fn resolve(&self, specifier: &str, from: &Path) -> RuntimeResult<PathBuf>;

    /// Create a directory
    async fn create_dir(&self, path: &Path, recursive: bool) -> RuntimeResult<()>;

    /// Remove a file
    async fn remove_file(&self, path: &Path) -> RuntimeResult<()>;

    /// Read a directory
    async fn read_dir(&self, path: &Path) -> RuntimeResult<Vec<String>>;

    /// Get the current working directory
    ///
    /// # Educational Note: Virtual Working Directory
    ///
    /// On WASM platforms, there is no OS-level current working directory.
    /// This method allows the runtime to provide a virtual cwd, enabling
    /// path resolution to work consistently across platforms.
    fn get_cwd(&self) -> RuntimeResult<PathBuf>;
}

// Native target: Multi-threaded, requires Send + Sync
#[cfg(not(target_family = "wasm"))]
#[async_trait]
pub trait Runtime: Send + Sync + std::fmt::Debug {
    /// Read a file from the filesystem
    async fn read_file(&self, path: &Path) -> RuntimeResult<Vec<u8>>;

    /// Write a file to the filesystem
    async fn write_file(&self, path: &Path, content: &[u8]) -> RuntimeResult<()>;

    /// Get file metadata
    async fn metadata(&self, path: &Path) -> RuntimeResult<FileMetadata>;

    /// Check if a path exists
    fn exists(&self, path: &Path) -> bool;

    /// Resolve a module specifier
    fn resolve(&self, specifier: &str, from: &Path) -> RuntimeResult<PathBuf>;

    /// Create a directory
    async fn create_dir(&self, path: &Path, recursive: bool) -> RuntimeResult<()>;

    /// Remove a file
    async fn remove_file(&self, path: &Path) -> RuntimeResult<()>;

    /// Read a directory
    async fn read_dir(&self, path: &Path) -> RuntimeResult<Vec<String>>;

    /// Get the current working directory
    ///
    /// # Educational Note: Working Directory on Native
    ///
    /// On native platforms, this delegates to `std::env::current_dir()`.
    /// This abstraction allows the bundler to work without directly calling
    /// OS-specific functions, making the code platform-agnostic.
    fn get_cwd(&self) -> RuntimeResult<PathBuf>;
}

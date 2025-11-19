//! Test utilities for fob-core.
//!
//! This module provides shared test infrastructure that works across all tests
//! in the fob-core crate. It's only available in test builds.
//!
//! ## Key Components
//!
//! - `TestRuntime`: A simple filesystem runtime for testing that wraps `std::fs`
//!
//! ## Educational Note: Test-Only Code Organization
//!
//! By centralizing test utilities:
//! 1. We eliminate code duplication across test modules
//! 2. We ensure consistent test behavior
//! 3. We make it easier to add new test helpers
//! 4. We keep the test runtime implementation in sync
//!
//! This module is only compiled when running tests (cfg(test)), so it doesn't
//! add any overhead to production builds.

// Test utilities are allowed to use std::fs since they only run on native platforms
#![allow(clippy::disallowed_methods)]

use crate::{FileMetadata, Runtime, RuntimeError, RuntimeResult};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Simple test runtime that wraps std::fs for native tests.
///
/// This allows tests to use real filesystem operations while still testing
/// the async Runtime API. It's designed to make testing straightforward by:
///
/// 1. Providing real filesystem access for integration tests
/// 2. Supporting all Runtime trait methods
/// 3. Being simple and predictable
///
/// ## Educational Note: Why TestRuntime?
///
/// Rather than mocking filesystem operations, we use real filesystem access
/// in a temporary directory (via `tempfile::TempDir`). This approach:
///
/// - Tests real I/O behavior including edge cases
/// - Validates path canonicalization works correctly
/// - Ensures security checks operate on real paths
/// - Makes tests more representative of production behavior
///
/// ## Usage Example
///
/// ```rust
/// use tempfile::TempDir;
/// use fob::test_utils::TestRuntime;
/// use std::fs;
///
/// # #[tokio::test]
/// # async fn example() {
/// let temp = TempDir::new().unwrap();
/// let cwd = temp.path().to_path_buf();
/// let runtime = TestRuntime::new(cwd.clone());
///
/// // Create test files
/// fs::write(cwd.join("test.txt"), b"content").unwrap();
///
/// // Use runtime in tests
/// let content = runtime.read_file(&cwd.join("test.txt")).await.unwrap();
/// assert_eq!(content, b"content");
/// # }
/// ```
#[cfg(not(target_family = "wasm"))]
#[derive(Debug)]
pub struct TestRuntime {
    cwd: PathBuf,
}

#[cfg(not(target_family = "wasm"))]
impl TestRuntime {
    /// Create a new test runtime with the specified working directory.
    ///
    /// # Arguments
    ///
    /// * `cwd` - The current working directory for path resolution
    ///
    /// # Example
    ///
    /// ```rust
    /// use tempfile::TempDir;
    /// use fob::test_utils::TestRuntime;
    ///
    /// let temp = TempDir::new().unwrap();
    /// let runtime = TestRuntime::new(temp.path().to_path_buf());
    /// ```
    pub fn new(cwd: PathBuf) -> Self {
        Self { cwd }
    }
}

#[cfg(not(target_family = "wasm"))]
#[async_trait]
impl Runtime for TestRuntime {
    async fn read_file(&self, path: &Path) -> RuntimeResult<Vec<u8>> {
        std::fs::read(path).map_err(|e| RuntimeError::Io(e.to_string()))
    }

    async fn write_file(&self, path: &Path, content: &[u8]) -> RuntimeResult<()> {
        std::fs::write(path, content).map_err(|e| RuntimeError::Io(e.to_string()))
    }

    async fn metadata(&self, path: &Path) -> RuntimeResult<FileMetadata> {
        let metadata = std::fs::metadata(path).map_err(|e| RuntimeError::Io(e.to_string()))?;
        Ok(FileMetadata {
            size: metadata.len(),
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
            modified: metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64),
        })
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn resolve(&self, specifier: &str, from: &Path) -> RuntimeResult<PathBuf> {
        Ok(from.parent().unwrap_or(&self.cwd).join(specifier))
    }

    async fn create_dir(&self, path: &Path, recursive: bool) -> RuntimeResult<()> {
        if recursive {
            std::fs::create_dir_all(path).map_err(|e| RuntimeError::Io(e.to_string()))
        } else {
            std::fs::create_dir(path).map_err(|e| RuntimeError::Io(e.to_string()))
        }
    }

    async fn remove_file(&self, path: &Path) -> RuntimeResult<()> {
        std::fs::remove_file(path).map_err(|e| RuntimeError::Io(e.to_string()))
    }

    async fn read_dir(&self, path: &Path) -> RuntimeResult<Vec<String>> {
        let entries: Vec<String> = std::fs::read_dir(path)
            .map_err(|e| RuntimeError::Io(e.to_string()))?
            .filter_map(|entry| {
                entry
                    .ok()
                    .and_then(|e| e.file_name().to_str().map(String::from))
            })
            .collect();
        Ok(entries)
    }

    fn get_cwd(&self) -> RuntimeResult<PathBuf> {
        Ok(self.cwd.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_runtime_read_write() {
        let temp = TempDir::new().unwrap();
        let cwd = temp.path().to_path_buf();
        let runtime = TestRuntime::new(cwd.clone());

        let file_path = cwd.join("test.txt");

        // Test write
        runtime
            .write_file(&file_path, b"hello world")
            .await
            .unwrap();

        // Test read
        let content = runtime.read_file(&file_path).await.unwrap();
        assert_eq!(content, b"hello world");
    }

    #[tokio::test]
    async fn test_runtime_metadata() {
        let temp = TempDir::new().unwrap();
        let cwd = temp.path().to_path_buf();
        let runtime = TestRuntime::new(cwd.clone());

        let file_path = cwd.join("test.txt");
        fs::write(&file_path, b"content").unwrap();

        let metadata = runtime.metadata(&file_path).await.unwrap();
        assert_eq!(metadata.size, 7);
        assert!(metadata.is_file);
        assert!(!metadata.is_dir);
    }

    #[tokio::test]
    async fn test_runtime_exists() {
        let temp = TempDir::new().unwrap();
        let cwd = temp.path().to_path_buf();
        let runtime = TestRuntime::new(cwd.clone());

        let file_path = cwd.join("test.txt");
        assert!(!runtime.exists(&file_path));

        fs::write(&file_path, b"content").unwrap();
        assert!(runtime.exists(&file_path));
    }

    #[tokio::test]
    async fn test_runtime_create_dir() {
        let temp = TempDir::new().unwrap();
        let cwd = temp.path().to_path_buf();
        let runtime = TestRuntime::new(cwd.clone());

        let dir_path = cwd.join("subdir");
        runtime.create_dir(&dir_path, false).await.unwrap();
        assert!(dir_path.exists());
        assert!(dir_path.is_dir());
    }

    #[tokio::test]
    async fn test_runtime_create_dir_recursive() {
        let temp = TempDir::new().unwrap();
        let cwd = temp.path().to_path_buf();
        let runtime = TestRuntime::new(cwd.clone());

        let nested_path = cwd.join("a/b/c");
        runtime.create_dir(&nested_path, true).await.unwrap();
        assert!(nested_path.exists());
        assert!(nested_path.is_dir());
    }

    #[tokio::test]
    async fn test_runtime_read_dir() {
        let temp = TempDir::new().unwrap();
        let cwd = temp.path().to_path_buf();
        let runtime = TestRuntime::new(cwd.clone());

        // Create some files
        fs::write(cwd.join("file1.txt"), b"1").unwrap();
        fs::write(cwd.join("file2.txt"), b"2").unwrap();
        fs::create_dir(cwd.join("subdir")).unwrap();

        let entries = runtime.read_dir(&cwd).await.unwrap();
        assert_eq!(entries.len(), 3);
        assert!(entries.contains(&"file1.txt".to_string()));
        assert!(entries.contains(&"file2.txt".to_string()));
        assert!(entries.contains(&"subdir".to_string()));
    }
}

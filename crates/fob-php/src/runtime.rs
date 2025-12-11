//! Native runtime implementation for PHP

use async_trait::async_trait;
use fob_bundler::{FileMetadata, Runtime, RuntimeError, RuntimeResult};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Native runtime for PHP using tokio
#[derive(Debug, Clone)]
pub struct NativeRuntime {
    resolver: Arc<oxc_resolver::Resolver>,
    cwd: PathBuf,
}

impl NativeRuntime {
    /// Create a new native runtime
    ///
    /// # Arguments
    ///
    /// * `cwd` - Working directory for resolution
    pub fn new(cwd: PathBuf) -> RuntimeResult<Self> {
        let resolver = oxc_resolver::Resolver::new(oxc_resolver::ResolveOptions {
            condition_names: vec!["import".into(), "module".into(), "default".into()],
            extensions: vec![".js".into(), ".ts".into(), ".jsx".into(), ".tsx".into()],
            ..Default::default()
        });

        Ok(Self {
            resolver: Arc::new(resolver),
            cwd,
        })
    }

    /// Get the current working directory for this runtime.
    pub fn cwd(&self) -> &Path {
        &self.cwd
    }
}

#[async_trait]
impl Runtime for NativeRuntime {
    async fn read_file(&self, path: &Path) -> RuntimeResult<Vec<u8>> {
        tokio::fs::read(path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                RuntimeError::FileNotFound(path.to_path_buf())
            } else {
                RuntimeError::Io(format!("Failed to read {}: {}", path.display(), e))
            }
        })
    }

    async fn write_file(&self, path: &Path, content: &[u8]) -> RuntimeResult<()> {
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                RuntimeError::Io(format!(
                    "Failed to create directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        tokio::fs::write(path, content)
            .await
            .map_err(|e| RuntimeError::Io(format!("Failed to write {}: {}", path.display(), e)))
    }

    async fn metadata(&self, path: &Path) -> RuntimeResult<FileMetadata> {
        let metadata = tokio::fs::metadata(path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                RuntimeError::FileNotFound(path.to_path_buf())
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

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn resolve(&self, specifier: &str, from: &Path) -> RuntimeResult<PathBuf> {
        let from_dir = if from.is_dir() {
            from
        } else {
            from.parent().unwrap_or(from)
        };

        self.resolver
            .resolve(from_dir, specifier)
            .map(|res| res.path().to_path_buf())
            .map_err(|e| RuntimeError::ResolutionFailed {
                specifier: specifier.to_string(),
                from: from.to_path_buf(),
                reason: format!("{:?}", e),
            })
    }

    async fn create_dir(&self, path: &Path, recursive: bool) -> RuntimeResult<()> {
        let result = if recursive {
            tokio::fs::create_dir_all(path).await
        } else {
            tokio::fs::create_dir(path).await
        };

        result.map_err(|e| {
            RuntimeError::Io(format!(
                "Failed to create directory {}: {}",
                path.display(),
                e
            ))
        })
    }

    async fn remove_file(&self, path: &Path) -> RuntimeResult<()> {
        tokio::fs::remove_file(path).await.map_err(|e| {
            RuntimeError::Io(format!("Failed to remove file {}: {}", path.display(), e))
        })
    }

    async fn read_dir(&self, path: &Path) -> RuntimeResult<Vec<String>> {
        let mut entries = tokio::fs::read_dir(path).await.map_err(|e| {
            RuntimeError::Io(format!(
                "Failed to read directory {}: {}",
                path.display(),
                e
            ))
        })?;

        let mut result = Vec::new();
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| RuntimeError::Io(format!("Failed to read directory entry: {}", e)))?
        {
            if let Some(name) = entry.file_name().to_str() {
                result.push(name.to_string());
            }
        }

        Ok(result)
    }

    fn get_cwd(&self) -> RuntimeResult<PathBuf> {
        Ok(self.cwd().to_path_buf())
    }
}

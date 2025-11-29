//! Extension resolution for module files.
//!
//! This module handles trying different file extensions (.ts, .js, .tsx, .jsx, etc.)
//! when resolving module imports.

use std::path::{Path, PathBuf};

use fob_core::runtime::{Runtime, RuntimeError};

use crate::config::ResolveResult;

/// Supported file extensions for module resolution.
pub const EXTENSIONS: &[&str] = &["ts", "tsx", "js", "jsx", "mjs", "json"];

/// Try to resolve a path with various extensions.
///
/// Returns the first matching file found, or None if no file exists.
pub async fn try_extensions(
    base_path: &Path,
    runtime: &dyn Runtime,
) -> Result<Option<PathBuf>, RuntimeError> {
    // First, try the path as-is (might already have extension)
    if runtime.exists(base_path) {
        if let Ok(metadata) = runtime.metadata(base_path).await {
            if metadata.is_file {
                return Ok(Some(base_path.to_path_buf()));
            }
        }
    }

    // Try with each extension
    for ext in EXTENSIONS {
        let with_ext = base_path.with_extension(ext);
        if runtime.exists(&with_ext) {
            if let Ok(metadata) = runtime.metadata(&with_ext).await {
                if metadata.is_file {
                    return Ok(Some(with_ext));
                }
            }
        }
    }

    Ok(None)
}

/// Try to resolve a directory with index files.
///
/// Returns the first matching index file found, or None if no index file exists.
pub async fn try_index_files(
    dir_path: &Path,
    runtime: &dyn Runtime,
) -> Result<Option<PathBuf>, RuntimeError> {
    if !runtime.exists(dir_path) {
        return Ok(None);
    }

    if let Ok(metadata) = runtime.metadata(dir_path).await {
        if metadata.is_dir {
            for ext in EXTENSIONS {
                let index = dir_path.join(format!("index.{}", ext));
                if runtime.exists(&index) {
                    return Ok(Some(index));
                }
            }
        }
    }

    Ok(None)
}

/// Resolve a local file path with extension and index file fallbacks.
pub async fn resolve_with_extensions(
    candidate: PathBuf,
    runtime: &dyn Runtime,
) -> Result<ResolveResult, RuntimeError> {
    // Try extensions first
    if let Some(resolved) = try_extensions(&candidate, runtime).await? {
        return Ok(ResolveResult::Local(resolved));
    }

    // Try as directory with index files
    if let Some(resolved) = try_index_files(&candidate, runtime).await? {
        return Ok(ResolveResult::Local(resolved));
    }

    // Could not resolve
    Ok(ResolveResult::Unresolved(
        candidate.to_string_lossy().to_string(),
    ))
}

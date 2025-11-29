//! Core resolution algorithm for module resolution.
//!
//! This module implements the main resolution logic that orchestrates
//! path alias resolution, extension resolution, and external package detection.

use std::path::Path;

use crate::runtime::{Runtime, RuntimeError};
use path_clean::PathClean;

use super::aliases::resolve_path_alias;
use super::extensions::{resolve_with_extensions, try_extensions, try_index_files};
use crate::analysis::config::ResolveResult;

/// Check if a specifier is explicitly marked as external.
pub fn is_external(specifier: &str, external: &[String]) -> bool {
    // Check exact match
    if external.iter().any(|ext| ext == specifier) {
        return true;
    }

    // Check if specifier starts with any external prefix
    // e.g., "react" matches external "react"
    // e.g., "react-dom" matches external "react"
    for ext in external {
        if specifier == ext || specifier.starts_with(&format!("{ext}/")) {
            return true;
        }
    }

    false
}

/// Resolve a local file path (relative or absolute).
pub async fn resolve_local(
    specifier: &str,
    from: &Path,
    cwd: Option<&Path>,
    runtime: &dyn Runtime,
) -> Result<ResolveResult, RuntimeError> {
    let base = if specifier.starts_with('/') {
        // Absolute path - use cwd as base if available
        cwd.unwrap_or_else(|| from.parent().unwrap_or(Path::new("")))
    } else {
        // Relative path
        from.parent().unwrap_or(Path::new(""))
    };

    // Join and normalize the path to handle . and .. components
    let candidate = base.join(specifier).clean();

    resolve_with_extensions(candidate, runtime).await
}

/// Resolve a module specifier using path aliases.
pub async fn resolve_with_alias(
    specifier: &str,
    from: &Path,
    cwd: &Path,
    path_aliases: &rustc_hash::FxHashMap<String, String>,
    runtime: &dyn Runtime,
) -> Result<Option<ResolveResult>, RuntimeError> {
    if let Some(resolved) = resolve_path_alias(specifier, from, path_aliases) {
        // Path aliases are resolved relative to cwd
        let base = if resolved.starts_with('/') {
            Path::new("")
        } else {
            cwd
        };
        let candidate = base.join(&resolved).clean();

        // Try with extensions
        if let Some(resolved_path) = try_extensions(&candidate, runtime).await? {
            return Ok(Some(ResolveResult::Local(resolved_path)));
        }

        // Try as directory with index files
        if let Some(resolved_path) = try_index_files(&candidate, runtime).await? {
            return Ok(Some(ResolveResult::Local(resolved_path)));
        }
    }

    Ok(None)
}

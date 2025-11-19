use crate::{Error, Result, Runtime};
use path_clean::PathClean;
use std::path::{Component, Path, PathBuf};

/// Validate that an asset path is safe to access.
///
/// # Security Checks
///
/// 1. Must not contain ".." components after canonicalization
/// 2. Must be within project directory, node_modules, or monorepo workspace
/// 3. Must not be a symlink to outside allowed directories
///
/// # Monorepo Support
///
/// In monorepo setups, workspace packages can reference assets in sibling packages.
/// We detect monorepo roots by looking for workspace metadata (pnpm-workspace.yaml,
/// lerna.json, or package.json with workspaces field) and allow access to files
/// within the monorepo root.
pub async fn validate_asset_security(
    canonical: &Path,
    cwd: &Path,
    runtime: &dyn Runtime,
) -> Result<()> {
    // Get canonicalized cwd
    let cwd_canonical =
        super::canonicalize_path(cwd, runtime)
            .await
            .map_err(|e| Error::IoError {
                message: "Failed to canonicalize working directory".to_string(),
                source: std::io::Error::other(format!("{e}")),
            })?;

    // Check if path is within project directory
    if canonical.starts_with(&cwd_canonical) {
        return Ok(());
    }

    // Check if path is within node_modules
    // Look for "node_modules" component in path
    let has_node_modules = canonical
        .components()
        .any(|c| matches!(c, Component::Normal(s) if s == "node_modules"));

    if has_node_modules {
        // Ensure it's still under a node_modules that's under cwd
        // Walk up from canonical to find node_modules, then check if that's under cwd
        let mut current = canonical;
        while let Some(parent) = current.parent() {
            if parent.file_name().and_then(|s| s.to_str()) == Some("node_modules") {
                if let Some(nm_parent) = parent.parent() {
                    if nm_parent.starts_with(&cwd_canonical) {
                        return Ok(());
                    }
                }
            }
            current = parent;
        }
    }

    // Check if we're in a monorepo and the asset is in a workspace package
    if let Some(monorepo_root) = find_monorepo_root(&cwd_canonical, runtime).await {
        if canonical.starts_with(&monorepo_root) {
            return Ok(());
        }
    }

    // Path is outside allowed directories
    Err(Error::AssetSecurityViolation {
        path: canonical.display().to_string(),
        reason: "Asset path is outside project directory and node_modules".to_string(),
    })
}

/// Find the monorepo root by walking up from cwd.
///
/// A directory is considered a monorepo root if it contains:
/// - pnpm-workspace.yaml (pnpm workspaces)
/// - lerna.json (Lerna monorepo)
/// - package.json with "workspaces" field (npm/yarn workspaces)
pub async fn find_monorepo_root(cwd: &Path, runtime: &dyn Runtime) -> Option<PathBuf> {
    let mut current = cwd;

    loop {
        // Check for pnpm workspace
        if runtime.exists(&current.join("pnpm-workspace.yaml")) {
            return Some(current.to_path_buf());
        }

        // Check for Lerna
        if runtime.exists(&current.join("lerna.json")) {
            return Some(current.to_path_buf());
        }

        // Check for npm/yarn workspaces in package.json
        if let Ok(content) = runtime.read_file(&current.join("package.json")).await {
            if let Ok(pkg_json) = String::from_utf8(content) {
                if pkg_json.contains("\"workspaces\"") {
                    return Some(current.to_path_buf());
                }
            }
        }

        // Move up to parent directory
        current = current.parent()?;
    }
}

/// Canonicalize a path using the runtime.
///
/// # Platform Differences
///
/// - **Native**: Tries real symlink resolution via `std::fs::canonicalize()` first,
///   falls back to path normalization if that fails (e.g., path doesn't exist yet)
/// - **WASM**: Uses path normalization only (no symlinks exist in virtual filesystem)
///
/// # Path Normalization
///
/// Uses the `path-clean` crate to properly resolve `.` and `..` components,
/// ensuring consistent path handling across platforms.
pub(crate) async fn canonicalize_path(path: &Path, runtime: &dyn Runtime) -> Result<PathBuf> {
    if !runtime.exists(path) {
        return Err(Error::AssetNotFound {
            specifier: path.display().to_string(),
            searched_from: "".to_string(),
        });
    }

    // Try real canonicalize on native platforms (resolves symlinks)
    #[cfg(not(target_family = "wasm"))]
    {
        if let Ok(canonical) = path.canonicalize() {
            return Ok(canonical);
        }
    }

    // Fallback: normalize path using path-clean
    // This handles . and .. components correctly
    Ok(path.to_path_buf().clean())
}

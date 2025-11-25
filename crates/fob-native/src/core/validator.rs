//! Path validation utilities

use path_clean::PathClean;
use std::path::{Component, Path, PathBuf};

/// Validate a path to prevent directory traversal attacks
///
/// Ensures the path is within the project root and doesn't contain
/// dangerous components like ".." or absolute paths outside the project.
pub fn validate_path(base: &Path, path: &Path, field_name: &str) -> Result<PathBuf, String> {
    // Canonicalize base first for consistent comparisons
    let canonical_base = base
        .canonicalize()
        .map_err(|e| format!("Base directory '{}' is invalid: {}", base.display(), e))?;

    // Resolve relative paths against base
    let resolved = if path.is_absolute() {
        // For absolute paths, canonicalize first to handle symlinks (e.g., /var -> /private/var on macOS)
        let canonical_path = if path.exists() {
            path.canonicalize().map_err(|e| {
                format!("{} path '{}' is invalid: {}", field_name, path.display(), e)
            })?
        } else if let Some(parent) = path.parent() {
            // Path doesn't exist - try to canonicalize parent to handle symlinks
            if parent.exists() {
                let canonical_parent = parent.canonicalize().map_err(|e| {
                    format!(
                        "{} path '{}' parent is invalid: {}",
                        field_name,
                        path.display(),
                        e
                    )
                })?;

                // Get the relative path from parent to the full path
                // This preserves all intermediate components, not just the filename
                // Since parent came from path.parent(), strip_prefix should always succeed
                let relative_from_parent = path.strip_prefix(parent).map_err(|_| {
                    format!(
                        "{} path '{}' is not a child of its parent '{}'",
                        field_name,
                        path.display(),
                        parent.display()
                    )
                })?;
                canonical_parent.join(relative_from_parent)
            } else {
                // Parent doesn't exist either - use cleaned path
                path.clean()
            }
        } else {
            // No parent - use cleaned path
            path.clean()
        };

        // Ensure canonicalized path is within the base directory
        if !canonical_path.starts_with(&canonical_base) {
            return Err(format!(
                "{} path '{}' is outside project directory '{}'",
                field_name,
                path.display(),
                canonical_base.display()
            ));
        }
        canonical_path
    } else {
        canonical_base.join(path)
    };

    // Normalize the path (resolves ".." and "." components)
    // For output directories, the path may not exist yet, so we handle that case
    let normalized = if resolved.exists() {
        resolved
            .canonicalize()
            .map_err(|e| format!("{} path '{}' is invalid: {}", field_name, path.display(), e))?
    } else {
        // Path doesn't exist yet - normalize without requiring existence
        // This is needed for output directories that will be created

        // Use path-clean to normalize . and .. components
        let cleaned = resolved.clean();

        // For additional safety on paths with "..", verify parent chain
        // Check if the path attempts to escape via ".."
        if path.components().any(|c| c == Component::ParentDir) {
            // If original path had "..", ensure cleaned version is still within base
            if !cleaned.starts_with(&canonical_base) {
                return Err(format!(
                    "{} path '{}' resolves outside project directory '{}' (directory traversal attempt)",
                    field_name,
                    path.display(),
                    canonical_base.display()
                ));
            }
        }

        cleaned
    };

    // Final check: ensure normalized path is still within base
    if !normalized.starts_with(&canonical_base) {
        return Err(format!(
            "{} path '{}' resolves outside project directory '{}' (possible directory traversal)",
            field_name,
            path.display(),
            canonical_base.display()
        ));
    }

    Ok(normalized)
}

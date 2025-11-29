//! Path validation and security checks for graph traversal.
//!
//! This module provides security checks to prevent path traversal attacks
//! and ensure that all resolved paths stay within the intended directory.

use std::path::{Path, PathBuf};
use thiserror::Error;

/// Error indicating a path traversal attempt was detected.
#[derive(Debug, Error)]
#[error("Path traversal detected: path '{path}' escapes from cwd '{cwd}'")]
pub struct PathTraversalError {
    /// The path that attempted to escape
    pub path: PathBuf,
    /// The current working directory that was escaped from
    pub cwd: PathBuf,
}

/// Validate that a normalized path stays within the current working directory.
///
/// This prevents path traversal attacks where malicious paths like `../../../etc/passwd`
/// could escape the intended directory.
///
/// # Arguments
///
/// * `normalized_path` - The normalized absolute path to validate
/// * `cwd` - The current working directory that serves as the root
///
/// # Returns
///
/// Ok(()) if the path is safe, or PathTraversalError if it escapes the cwd
pub fn validate_path_within_cwd(
    normalized_path: &Path,
    cwd: &Path,
) -> Result<(), PathTraversalError> {
    // First, try with canonicalized paths (most accurate)
    if let (Ok(normalized), Ok(cwd_normalized)) =
        (normalized_path.canonicalize(), cwd.canonicalize())
    {
        if normalized.starts_with(&cwd_normalized) {
            return Ok(());
        }
    }

    // Fallback: check with non-canonicalized paths
    // This handles cases where paths don't exist yet or canonicalization fails
    // We use a more lenient check here, but still validate the path structure
    if normalized_path.starts_with(cwd) {
        // Additional check: ensure the path doesn't contain ".." that would escape
        // This is a safety check for the fallback case
        // Count how many directory levels we're going up from cwd
        let relative = normalized_path
            .strip_prefix(cwd)
            .ok()
            .and_then(|p| p.to_str());

        if let Some(rel) = relative {
            // Check if the relative path contains excessive ".."
            let dot_dot_count = rel.matches("../").count();
            if dot_dot_count == 0 {
                // No ".." components, safe
                return Ok(());
            }
        }

        // If we have ".." components, be more strict
        // Only allow if canonicalization succeeded and passed the starts_with check above
        // Otherwise, reject to be safe
        return Err(PathTraversalError {
            path: normalized_path.to_path_buf(),
            cwd: cwd.to_path_buf(),
        });
    }

    // Path doesn't start with cwd, definitely a traversal attempt
    Err(PathTraversalError {
        path: normalized_path.to_path_buf(),
        cwd: cwd.to_path_buf(),
    })
}

/// Normalize a path and validate it stays within the cwd.
///
/// This is a secure version of path normalization that:
/// 1. Converts relative paths to absolute paths
/// 2. Cleans the path (removes `.` and `..` components)
/// 3. Validates the path doesn't escape the cwd
///
/// # Arguments
///
/// * `path` - The path to normalize (can be relative or absolute)
/// * `cwd` - The current working directory
///
/// # Returns
///
/// The normalized absolute path, or an error if path traversal is detected
pub fn normalize_and_validate_path(path: &Path, cwd: &Path) -> Result<PathBuf, PathTraversalError> {
    use path_clean::PathClean;

    let normalized = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };

    let cleaned = normalized.clean();

    // Validate the cleaned path stays within cwd
    validate_path_within_cwd(&cleaned, cwd)?;

    Ok(cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_path_within_cwd_valid() {
        let temp_dir = TempDir::new().unwrap();
        let cwd = temp_dir.path();
        let valid_path = cwd.join("src").join("index.ts");

        // Create the directory structure
        fs::create_dir_all(valid_path.parent().unwrap()).unwrap();
        fs::write(&valid_path, "").unwrap();

        assert!(validate_path_within_cwd(&valid_path, cwd).is_ok());
    }

    #[test]
    fn test_validate_path_within_cwd_traversal() {
        let temp_dir = TempDir::new().unwrap();
        let cwd = temp_dir.path();
        let traversal_path = cwd.join("..").join("etc").join("passwd");

        let result = validate_path_within_cwd(&traversal_path, cwd);
        assert!(result.is_err());
        if let Err(PathTraversalError { path, cwd: _ }) = result {
            assert_eq!(path, traversal_path);
        }
    }

    #[test]
    fn test_normalize_and_validate_relative_path() {
        let temp_dir = TempDir::new().unwrap();
        let cwd = temp_dir.path();
        let relative_path = Path::new("src/index.ts");
        let expected = cwd.join("src").join("index.ts");

        let result = normalize_and_validate_path(relative_path, cwd).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_normalize_and_validate_traversal_attempt() {
        let temp_dir = TempDir::new().unwrap();
        let cwd = temp_dir.path();
        let traversal_path = Path::new("../../../etc/passwd");

        let result = normalize_and_validate_path(traversal_path, cwd);
        assert!(result.is_err());
    }

    #[test]
    fn test_normalize_and_validate_with_dot_components() {
        let temp_dir = TempDir::new().unwrap();
        let cwd = temp_dir.path();
        let path_with_dots = Path::new("./src/../src/./index.ts");
        let expected = cwd.join("src").join("index.ts");

        let result = normalize_and_validate_path(path_with_dots, cwd).unwrap();
        assert_eq!(result, expected);
    }
}

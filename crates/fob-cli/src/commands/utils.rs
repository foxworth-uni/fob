//! Shared utilities for command implementations.
//!
//! This module provides common functionality used across multiple commands:
//!
//! - Path resolution and validation
//! - Configuration loading and merging
//! - Directory cleaning operations
//! - Entry point validation
//! - Package manager detection

use crate::error::{BuildError, CliError, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Resolve a path relative to a working directory.
///
/// If the path is absolute, returns it unchanged. Otherwise, joins it with
/// the working directory.
///
/// # Arguments
///
/// * `path` - Path to resolve
/// * `cwd` - Working directory to resolve relative to
///
/// # Returns
///
/// Absolute path
pub fn resolve_path(path: &Path, cwd: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

/// Validate that an entry point file exists.
///
/// # Arguments
///
/// * `entry` - Path to entry point file
///
/// # Errors
///
/// Returns `BuildError::EntryNotFound` if the file doesn't exist.
pub fn validate_entry(entry: &Path) -> Result<()> {
    if !entry.exists() {
        return Err(BuildError::EntryNotFound(entry.to_path_buf()).into());
    }

    if !entry.is_file() {
        return Err(CliError::InvalidArgument(format!(
            "Entry point is not a file: {}",
            entry.display()
        )));
    }

    Ok(())
}

/// Clean an output directory by removing all its contents.
///
/// Creates the directory if it doesn't exist. If it exists, removes all files
/// and subdirectories within it.
///
/// # Arguments
///
/// * `out_dir` - Directory to clean
///
/// # Errors
///
/// Returns I/O errors if directory operations fail.
///
/// # Safety
///
/// This function performs destructive filesystem operations. It validates that
/// the path exists and is actually a directory before removing contents to
/// prevent accidental data loss.
pub fn clean_output_dir(out_dir: &Path) -> Result<()> {
    if out_dir.exists() {
        if !out_dir.is_dir() {
            return Err(CliError::InvalidArgument(format!(
                "Output path exists but is not a directory: {}",
                out_dir.display()
            )));
        }

        // Remove all contents but keep the directory itself
        for entry in fs::read_dir(out_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
        }
    } else {
        // Create directory if it doesn't exist
        fs::create_dir_all(out_dir)?;
    }

    Ok(())
}

/// Ensure an output directory exists, creating it if necessary.
///
/// # Arguments
///
/// * `out_dir` - Directory to ensure exists
///
/// # Errors
///
/// Returns I/O errors if directory creation fails.
pub fn ensure_output_dir(out_dir: &Path) -> Result<()> {
    if !out_dir.exists() {
        fs::create_dir_all(out_dir)?;
    } else if !out_dir.is_dir() {
        return Err(CliError::InvalidArgument(format!(
            "Output path exists but is not a directory: {}",
            out_dir.display()
        )));
    }

    Ok(())
}

/// Get the current working directory.
///
/// # Errors
///
/// Returns I/O error if current directory cannot be determined.
pub fn get_cwd() -> Result<PathBuf> {
    std::env::current_dir().map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to get current directory: {}", e),
        ))
    })
}

/// Detect which package manager is being used in a project.
///
/// Checks for lock files in order of preference: pnpm > yarn > npm.
///
/// # Arguments
///
/// * `project_dir` - Directory to check for lock files
///
/// # Returns
///
/// Package manager name ("pnpm", "yarn", or "npm")
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}

impl PackageManager {
    /// Detect package manager from lock files.
    ///
    /// Detection order (highest priority first):
    /// 1. `pnpm-lock.yaml` → pnpm
    /// 2. `yarn.lock` → yarn
    /// 3. `bun.lockb` → bun
    /// 4. Default to npm (also covers package-lock.json)
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn detect(project_dir: &Path) -> Self {
        if project_dir.join("pnpm-lock.yaml").exists() {
            PackageManager::Pnpm
        } else if project_dir.join("yarn.lock").exists() {
            PackageManager::Yarn
        } else if project_dir.join("bun.lockb").exists() {
            PackageManager::Bun
        } else {
            PackageManager::Npm
        }
    }

    /// Get the command name for this package manager.
    pub fn command(&self) -> &'static str {
        match self {
            PackageManager::Npm => "npm",
            PackageManager::Yarn => "yarn",
            PackageManager::Pnpm => "pnpm",
            PackageManager::Bun => "bun",
        }
    }

    /// Get the install command for this package manager.
    pub fn install_cmd(&self) -> &'static str {
        match self {
            PackageManager::Npm => "npm install",
            PackageManager::Yarn => "yarn install",
            PackageManager::Pnpm => "pnpm install",
            PackageManager::Bun => "bun install",
        }
    }
}

impl std::fmt::Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.command())
    }
}

/// Walks up the directory tree to find the nearest package.json.
///
/// Starts from `start_dir` and traverses parent directories until:
/// - A `package.json` file is found (returns the containing directory)
/// - The filesystem root is reached (returns None)
///
/// # Arguments
/// * `start_dir` - Directory to begin searching from
///
/// # Returns
/// * `Some(PathBuf)` - Directory containing package.json
/// * `None` - No package.json found up to filesystem root
///
/// # Examples
/// ```
/// # use std::path::Path;
/// # use fob_cli::commands::utils::find_package_json;
/// let root = find_package_json(Path::new("/project/src/components"));
/// // Returns Some("/project") if /project/package.json exists
/// ```
pub fn find_package_json(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir;

    loop {
        let package_json_path = current.join("package.json");

        if package_json_path.exists() && package_json_path.is_file() {
            return Some(current.to_path_buf());
        }

        // Try to move to parent directory
        match current.parent() {
            Some(parent) => current = parent,
            None => {
                // Reached filesystem root without finding package.json
                return None;
            }
        }
    }
}

/// Resolves the project root directory using smart auto-detection.
///
/// Resolution priority (highest to lowest):
/// 1. Explicit `--cwd` flag if provided
/// 2. Directory containing the entry point's package.json
/// 3. Nearest package.json walking up from process.cwd()
/// 4. Fallback to process.cwd() with warning
///
/// # Arguments
/// * `explicit_cwd` - Optional directory from `--cwd` flag
/// * `entry_point` - Optional entry file path for detection
///
/// # Returns
/// * `Ok(PathBuf)` - Resolved absolute project root path
/// * `Err(CliError)` - If explicit cwd is invalid or paths cannot be resolved
///
/// # Errors
/// - Explicit cwd doesn't exist or isn't a directory
/// - Cannot determine current working directory
///
/// # Examples
/// ```no_run
/// # use std::path::Path;
/// # use fob_cli::commands::utils::resolve_project_root;
/// // With explicit cwd (highest priority)
/// let root = resolve_project_root(Some(Path::new("/my/project")), None)?;
///
/// // With entry point detection
/// let root = resolve_project_root(None, Some("./src/index.ts"))?;
///
/// // Auto-detection from current directory
/// let root = resolve_project_root(None, None)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn resolve_project_root(
    explicit_cwd: Option<&Path>,
    entry_point: Option<&str>,
) -> Result<PathBuf> {
    use crate::ui;

    // Priority 1: Explicit --cwd flag (user override)
    if let Some(cwd_path) = explicit_cwd {
        let absolute = if cwd_path.is_absolute() {
            cwd_path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| CliError::Io(e))?
                .join(cwd_path)
        };

        if !absolute.exists() {
            return Err(CliError::InvalidArgument(format!(
                "Specified --cwd directory does not exist: {}",
                absolute.display()
            )));
        }

        if !absolute.is_dir() {
            return Err(CliError::InvalidArgument(format!(
                "Specified --cwd is not a directory: {}",
                absolute.display()
            )));
        }

        ui::info(&format!(
            "Using project root: {} (from --cwd flag)",
            absolute.display()
        ));
        return Ok(absolute);
    }

    // Priority 2: Entry point's package.json
    if let Some(entry) = entry_point {
        let current_dir = std::env::current_dir().map_err(|e| CliError::Io(e))?;

        let entry_path = if Path::new(entry).is_absolute() {
            PathBuf::from(entry)
        } else {
            current_dir.join(entry)
        };

        // Get the directory containing the entry file
        if let Some(entry_dir) = entry_path.parent() {
            if let Some(package_root) = find_package_json(entry_dir) {
                ui::info(&format!(
                    "Using project root: {} (detected from entry point's package.json)",
                    package_root.display()
                ));
                return Ok(package_root);
            }
        }
    }

    // Priority 3: Current directory's package.json
    let current_dir = std::env::current_dir().map_err(|e| CliError::Io(e))?;

    if let Some(package_root) = find_package_json(&current_dir) {
        ui::info(&format!(
            "Using project root: {} (auto-detected from package.json)",
            package_root.display()
        ));
        return Ok(package_root);
    }

    // Priority 4: Fallback to current directory with warning
    ui::warning(&format!(
        "No package.json found. Using current directory: {}",
        current_dir.display()
    ));
    ui::info("Consider using --cwd to specify your project root explicitly.");

    Ok(current_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_path_absolute() {
        let abs_path = PathBuf::from("/absolute/path");
        let cwd = PathBuf::from("/some/dir");

        let resolved = resolve_path(&abs_path, &cwd);
        assert_eq!(resolved, abs_path);
    }

    #[test]
    fn test_resolve_path_relative() {
        let rel_path = PathBuf::from("relative/path");
        let cwd = PathBuf::from("/some/dir");

        let resolved = resolve_path(&rel_path, &cwd);
        assert_eq!(resolved, PathBuf::from("/some/dir/relative/path"));
    }

    #[test]
    fn test_validate_entry_exists() {
        let temp_dir = TempDir::new().unwrap();
        let entry_path = temp_dir.path().join("index.ts");
        File::create(&entry_path).unwrap();

        assert!(validate_entry(&entry_path).is_ok());
    }

    #[test]
    fn test_validate_entry_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let entry_path = temp_dir.path().join("nonexistent.ts");

        let result = validate_entry(&entry_path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CliError::Build(BuildError::EntryNotFound { .. })
        ));
    }

    #[test]
    fn test_validate_entry_not_file() {
        let temp_dir = TempDir::new().unwrap();

        let result = validate_entry(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_clean_output_dir_creates_if_missing() {
        let temp_dir = TempDir::new().unwrap();
        let out_dir = temp_dir.path().join("dist");

        assert!(!out_dir.exists());
        clean_output_dir(&out_dir).unwrap();
        assert!(out_dir.exists());
        assert!(out_dir.is_dir());
    }

    #[test]
    fn test_clean_output_dir_removes_contents() {
        let temp_dir = TempDir::new().unwrap();
        let out_dir = temp_dir.path().join("dist");
        fs::create_dir(&out_dir).unwrap();

        // Create some files and directories
        File::create(out_dir.join("file1.js")).unwrap();
        File::create(out_dir.join("file2.js")).unwrap();
        fs::create_dir(out_dir.join("subdir")).unwrap();
        File::create(out_dir.join("subdir/file3.js")).unwrap();

        clean_output_dir(&out_dir).unwrap();

        // Directory should exist but be empty
        assert!(out_dir.exists());
        assert!(out_dir.is_dir());
        assert_eq!(fs::read_dir(&out_dir).unwrap().count(), 0);
    }

    #[test]
    fn test_clean_output_dir_not_a_directory() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("not_a_dir");
        File::create(&file_path).unwrap();

        let result = clean_output_dir(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_ensure_output_dir_creates() {
        let temp_dir = TempDir::new().unwrap();
        let out_dir = temp_dir.path().join("new_dir");

        ensure_output_dir(&out_dir).unwrap();
        assert!(out_dir.exists());
        assert!(out_dir.is_dir());
    }

    #[test]
    fn test_ensure_output_dir_exists() {
        let temp_dir = TempDir::new().unwrap();
        let out_dir = temp_dir.path().join("existing");
        fs::create_dir(&out_dir).unwrap();

        // Should succeed without error
        ensure_output_dir(&out_dir).unwrap();
    }

    #[test]
    fn test_get_cwd() {
        let cwd = get_cwd().unwrap();
        assert!(cwd.is_absolute());
    }

    #[test]
    fn test_package_manager_detect_pnpm() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("pnpm-lock.yaml")).unwrap();

        assert_eq!(
            PackageManager::detect(temp_dir.path()),
            PackageManager::Pnpm
        );
    }

    #[test]
    fn test_package_manager_detect_yarn() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("yarn.lock")).unwrap();

        assert_eq!(
            PackageManager::detect(temp_dir.path()),
            PackageManager::Yarn
        );
    }

    #[test]
    fn test_package_manager_detect_npm() {
        let temp_dir = TempDir::new().unwrap();
        // No lock file defaults to npm

        assert_eq!(PackageManager::detect(temp_dir.path()), PackageManager::Npm);
    }

    #[test]
    fn test_package_manager_pnpm_prefers_over_yarn() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("pnpm-lock.yaml")).unwrap();
        File::create(temp_dir.path().join("yarn.lock")).unwrap();

        // pnpm should win
        assert_eq!(
            PackageManager::detect(temp_dir.path()),
            PackageManager::Pnpm
        );
    }

    #[test]
    fn test_package_manager_commands() {
        assert_eq!(PackageManager::Npm.command(), "npm");
        assert_eq!(PackageManager::Yarn.command(), "yarn");
        assert_eq!(PackageManager::Pnpm.command(), "pnpm");
    }

    #[test]
    fn test_package_manager_install_cmd() {
        assert_eq!(PackageManager::Npm.install_cmd(), "npm install");
        assert_eq!(PackageManager::Yarn.install_cmd(), "yarn install");
        assert_eq!(PackageManager::Pnpm.install_cmd(), "pnpm install");
    }

    #[test]
    fn test_find_package_json_in_current_dir() {
        let temp = TempDir::new().unwrap();
        let package_json = temp.path().join("package.json");
        File::create(&package_json).unwrap();

        let result = find_package_json(temp.path());
        assert_eq!(result, Some(temp.path().to_path_buf()));
    }

    #[test]
    fn test_find_package_json_walks_up() {
        let temp = TempDir::new().unwrap();
        let package_json = temp.path().join("package.json");
        File::create(&package_json).unwrap();

        let nested = temp.path().join("src").join("components");
        fs::create_dir_all(&nested).unwrap();

        let result = find_package_json(&nested);
        assert_eq!(result, Some(temp.path().to_path_buf()));
    }

    #[test]
    fn test_find_package_json_stops_at_first() {
        let temp = TempDir::new().unwrap();

        // Create nested package.json files
        let root_package = temp.path().join("package.json");
        File::create(&root_package).unwrap();

        let nested = temp.path().join("packages").join("app");
        fs::create_dir_all(&nested).unwrap();
        let nested_package = nested.join("package.json");
        File::create(&nested_package).unwrap();

        // Should find the nearest one
        let result = find_package_json(&nested);
        assert_eq!(result, Some(nested.clone()));
    }

    #[test]
    fn test_resolve_project_root_explicit_cwd() {
        let temp = TempDir::new().unwrap();

        let result = resolve_project_root(Some(temp.path()), None).unwrap();
        assert_eq!(result, temp.path());
    }

    #[test]
    fn test_resolve_project_root_explicit_cwd_invalid() {
        let invalid_path = Path::new("/this/path/definitely/does/not/exist/12345");

        let result = resolve_project_root(Some(invalid_path), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_resolve_project_root_from_entry() {
        let temp = TempDir::new().unwrap();
        let package_json = temp.path().join("package.json");
        File::create(&package_json).unwrap();

        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        let entry = src.join("index.ts");
        File::create(&entry).unwrap();

        // Change to temp directory for relative path resolution
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = resolve_project_root(None, Some("src/index.ts")).unwrap();
        // Canonicalize both paths for comparison (handles macOS /var -> /private/var symlink)
        let expected = temp.path().canonicalize().unwrap();
        assert_eq!(result.canonicalize().unwrap(), expected);

        std::env::set_current_dir(original).unwrap();
    }

    #[test]
    fn test_resolve_project_root_fallback() {
        // When no package.json exists, should return current dir with warning
        let result = resolve_project_root(None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_package_manager_pnpm() {
        let temp = TempDir::new().unwrap();
        File::create(temp.path().join("pnpm-lock.yaml")).unwrap();

        assert_eq!(PackageManager::detect(temp.path()), PackageManager::Pnpm);
    }

    #[test]
    fn test_detect_package_manager_yarn() {
        let temp = TempDir::new().unwrap();
        File::create(temp.path().join("yarn.lock")).unwrap();

        assert_eq!(PackageManager::detect(temp.path()), PackageManager::Yarn);
    }

    #[test]
    fn test_detect_package_manager_bun() {
        let temp = TempDir::new().unwrap();
        File::create(temp.path().join("bun.lockb")).unwrap();

        assert_eq!(PackageManager::detect(temp.path()), PackageManager::Bun);
    }

    #[test]
    fn test_detect_package_manager_npm() {
        let temp = TempDir::new().unwrap();
        File::create(temp.path().join("package-lock.json")).unwrap();

        assert_eq!(PackageManager::detect(temp.path()), PackageManager::Npm);
    }

    #[test]
    fn test_detect_package_manager_default_npm() {
        let temp = TempDir::new().unwrap();
        // No lockfile - should default to npm

        assert_eq!(PackageManager::detect(temp.path()), PackageManager::Npm);
    }

    #[test]
    fn test_detect_package_manager_priority() {
        let temp = TempDir::new().unwrap();
        // Create multiple lockfiles - pnpm should win
        File::create(temp.path().join("pnpm-lock.yaml")).unwrap();
        File::create(temp.path().join("yarn.lock")).unwrap();
        File::create(temp.path().join("package-lock.json")).unwrap();

        assert_eq!(PackageManager::detect(temp.path()), PackageManager::Pnpm);
    }
}

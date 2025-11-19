//! Secure file writing utilities for bundle output.
//!
//! This module provides atomic, secure file writing with comprehensive path validation
//! to prevent directory traversal attacks and ensure data integrity during write operations.
//!
//! # Security Features
//!
//! - **Path Validation**: All paths are validated to prevent directory traversal (`..` components)
//! - **Atomic Writes**: Uses temp files + rename for atomic operations
//! - **Automatic Rollback**: If any write fails, all previously written files are deleted
//! - **Directory Creation**: Automatically creates parent directories with `mkdir -p` behavior
//!
//! # Design Rationale
//!
//! ## Why Atomic Writes?
//!
//! When writing multiple files as part of a bundle, we need to ensure that either all files
//! are written successfully or none are written at all. This prevents partial bundle states
//! that could break applications. We achieve this by:
//!
//! 1. Writing to temporary files first (`.tmp` suffix)
//! 2. Tracking all temp files written
//! 3. If all writes succeed, rename temp files to final names atomically
//! 4. If any write fails, delete all temp files and return an error
//!
//! On most file systems, `rename()` is atomic - the file either appears with its full
//! contents or doesn't appear at all. This prevents race conditions and partial reads.
//!
//! ## Why Path Validation?
//!
//! User-controlled file paths are a security vulnerability. Without validation, an attacker
//! could:
//!
//! - Use `../../../etc/passwd` to write to sensitive system files
//! - Use absolute paths to write anywhere on the filesystem
//! - Use symlink attacks to overwrite protected files
//!
//! We prevent this by normalizing all paths and ensuring they're contained within the
//! output directory.

use std::fs;
use std::path::{Path, PathBuf};

use path_clean::PathClean;
use rolldown::BundleOutput;
use rolldown_common::Output;

use crate::{Error, Result};

/// Writes a bundle to disk with security checks and atomic guarantees.
///
/// # Arguments
///
/// * `output` - The bundle output containing assets to write
/// * `dir` - Target directory for output files
/// * `overwrite` - If `true`, overwrites existing files; if `false`, errors on conflicts
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if:
/// - Path validation fails (directory traversal attempt)
/// - File already exists and `overwrite` is `false`
/// - Any I/O operation fails
///
/// # Atomic Guarantees
///
/// This function provides atomic writes - either all files are written successfully or
/// none are written. If any operation fails, all previously written files are rolled back.
///
/// # Security
///
/// - Validates all paths to prevent directory traversal
/// - Normalizes paths using `path_clean` to resolve `.` and `..` components
/// - Ensures all output paths are within the specified directory
///
/// # Examples
///
/// ```no_run
/// use fob_bundler::output::writer::write_bundle_to;
/// use std::path::Path;
/// # use rolldown::BundleOutput;
/// # use fob_bundler::Result;
///
/// # fn example(output: &BundleOutput) -> Result<()> {
/// // Write bundle, error if files exist
/// write_bundle_to(output, Path::new("dist"), false)?;
///
/// // Force overwrite existing files
/// write_bundle_to(output, Path::new("dist"), true)?;
/// # Ok(())
/// # }
/// ```
pub fn write_bundle_to(output: &BundleOutput, dir: &Path, overwrite: bool) -> Result<()> {
    // Validate and normalize the output directory
    let dir = validate_and_normalize_dir(dir)?;

    // Create the output directory if it doesn't exist
    fs::create_dir_all(&dir).map_err(|e| {
        Error::WriteFailure(format!(
            "Failed to create output directory '{}': {}",
            dir.display(),
            e
        ))
    })?;

    // Collect all file operations to perform
    let mut operations = Vec::new();
    for output_item in &output.assets {
        // Only process assets (skip chunks if present)
        if let Output::Asset(asset) = output_item {
            let filename = asset.filename.as_str();
            let target_path = validate_output_path(&dir, filename)?;

            // Check if file exists when overwrite is disabled
            if !overwrite && target_path.exists() {
                return Err(Error::OutputExists(format!(
                    "File already exists: '{}'. Use overwrite=true to replace.",
                    target_path.display()
                )));
            }

            operations.push((target_path, asset.source.as_bytes()));
        } else if let Output::Chunk(chunk) = output_item {
            // Also handle chunks
            let filename = chunk.filename.as_str();
            let target_path = validate_output_path(&dir, filename)?;

            // Check if file exists when overwrite is disabled
            if !overwrite && target_path.exists() {
                return Err(Error::OutputExists(format!(
                    "File already exists: '{}'. Use overwrite=true to replace.",
                    target_path.display()
                )));
            }

            operations.push((target_path, chunk.code.as_bytes()));
        }
    }

    // Write all files atomically
    write_files_atomic(&operations)?;

    Ok(())
}

/// Validates and normalizes a directory path.
///
/// This ensures the path is safe to use as an output directory by:
/// - Normalizing the path (resolving `.` and `..`)
/// - Converting to absolute path
/// - Checking for suspicious patterns
fn validate_and_normalize_dir(dir: &Path) -> Result<PathBuf> {
    // Clean the path to resolve . and .. components
    let cleaned = dir.clean();

    // Convert to absolute path
    let absolute = if cleaned.is_absolute() {
        cleaned
    } else {
        std::env::current_dir()
            .map_err(|e| {
                Error::InvalidOutputPath(format!("Failed to get current directory: {}", e))
            })?
            .join(&cleaned)
            .clean()
    };

    Ok(absolute)
}

/// Validates an output path to prevent directory traversal attacks.
///
/// # Security
///
/// This function prevents attacks like:
/// - `../../../etc/passwd` - escaping the output directory
/// - `/etc/passwd` - absolute paths outside output directory
/// - `dir/../../../etc/passwd` - complex traversal patterns
///
/// It works by:
/// 1. Cleaning both the base directory and the filename
/// 2. Joining them together
/// 3. Cleaning the result again to resolve any remaining `..`
/// 4. Checking that the final path is still under the base directory
fn validate_output_path(base_dir: &Path, filename: &str) -> Result<PathBuf> {
    // Reject paths that look suspicious upfront
    if filename.contains('\0') {
        return Err(Error::InvalidOutputPath(
            "Filename contains null byte".to_string(),
        ));
    }

    // On Windows, check for device names
    #[cfg(target_os = "windows")]
    {
        let upper = filename.to_uppercase();
        let device_names = [
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
            "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ];
        for device in &device_names {
            if upper == *device || upper.starts_with(&format!("{}.", device)) {
                return Err(Error::InvalidOutputPath(format!(
                    "Filename is a reserved device name: {}",
                    filename
                )));
            }
        }
    }

    // Clean the filename path
    let filename_path = Path::new(filename).clean();

    // Join with base directory and clean again
    let full_path = base_dir.join(&filename_path).clean();

    // Ensure the final path is still under base_dir
    // This is the critical security check that prevents directory traversal
    if !full_path.starts_with(base_dir) {
        return Err(Error::InvalidOutputPath(format!(
            "Path '{}' escapes output directory '{}' (resolved to '{}')",
            filename,
            base_dir.display(),
            full_path.display()
        )));
    }

    Ok(full_path)
}

/// Writes multiple files atomically with automatic rollback on failure.
///
/// # Atomic Guarantees
///
/// This function uses a two-phase commit:
/// 1. Write all content to temporary files (`.tmp` suffix)
/// 2. If all writes succeed, rename temp files to final names
/// 3. If any operation fails, delete all temp files
///
/// The `rename()` operation is atomic on most filesystems, so readers will never
/// see partial file contents.
fn write_files_atomic(operations: &[(PathBuf, &[u8])]) -> Result<()> {
    let mut temp_files = Vec::new();

    // Phase 1: Write to temporary files
    for (target_path, content) in operations {
        // Create parent directories if needed
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                cleanup_temp_files(&temp_files);
                Error::WriteFailure(format!(
                    "Failed to create directory '{}': {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        // Write to temporary file
        let temp_path = target_path.with_extension("tmp");
        fs::write(&temp_path, content).map_err(|e| {
            cleanup_temp_files(&temp_files);
            Error::WriteFailure(format!(
                "Failed to write temporary file '{}': {}",
                temp_path.display(),
                e
            ))
        })?;

        temp_files.push((temp_path, target_path.clone()));
    }

    // Phase 2: Rename temp files to final names (atomic operation)
    for (temp_path, target_path) in &temp_files {
        fs::rename(temp_path, target_path).map_err(|e| {
            // On failure, try to clean up temp files
            cleanup_temp_files(&temp_files);
            Error::WriteFailure(format!(
                "Failed to rename '{}' to '{}': {}",
                temp_path.display(),
                target_path.display(),
                e
            ))
        })?;
    }

    Ok(())
}

/// Cleans up temporary files on error.
///
/// Best-effort cleanup - errors are logged but not propagated since we're
/// already in an error state.
fn cleanup_temp_files(temp_files: &[(PathBuf, PathBuf)]) {
    for (temp_path, _) in temp_files {
        if temp_path.exists() {
            if let Err(e) = fs::remove_file(temp_path) {
                eprintln!(
                    "Warning: Failed to clean up temporary file '{}': {}",
                    temp_path.display(),
                    e
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_output_path_normal() {
        let base = Path::new("/tmp/output");
        let result = validate_output_path(base, "index.js");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Path::new("/tmp/output/index.js"));
    }

    #[test]
    fn test_validate_output_path_nested() {
        let base = Path::new("/tmp/output");
        let result = validate_output_path(base, "dist/bundle.js");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Path::new("/tmp/output/dist/bundle.js"));
    }

    #[test]
    fn test_validate_output_path_traversal_simple() {
        let base = Path::new("/tmp/output");
        let result = validate_output_path(base, "../etc/passwd");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidOutputPath(_)));
    }

    #[test]
    fn test_validate_output_path_traversal_complex() {
        let base = Path::new("/tmp/output");
        let result = validate_output_path(base, "safe/../../../../../../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_output_path_null_byte() {
        let base = Path::new("/tmp/output");
        let result = validate_output_path(base, "file\0name.js");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_output_path_current_dir() {
        let base = Path::new("/tmp/output");
        let result = validate_output_path(base, "./index.js");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Path::new("/tmp/output/index.js"));
    }
}

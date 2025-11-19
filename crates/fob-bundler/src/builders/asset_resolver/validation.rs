use crate::{Error, Result, Runtime, RuntimeError};
use std::path::Path;

/// Validate asset size to prevent DoS attacks.
///
/// # Arguments
///
/// * `path` - Path to the asset file
/// * `max_size` - Maximum allowed size in bytes (default: 50MB)
/// * `runtime` - Runtime for filesystem operations
///
/// # Returns
///
/// File size in bytes if valid
pub async fn validate_asset_size(
    path: &Path,
    max_size: Option<u64>,
    runtime: &dyn Runtime,
) -> Result<u64> {
    let max_size = max_size.unwrap_or(50 * 1024 * 1024); // 50MB default

    let metadata = runtime.metadata(path).await.map_err(|e| match e {
        RuntimeError::FileNotFound(p) => Error::AssetNotFound {
            specifier: p.display().to_string(),
            searched_from: "".to_string(),
        },
        RuntimeError::Io(msg) => Error::IoError {
            message: format!("Failed to read asset metadata: {}", path.display()),
            source: std::io::Error::other(msg),
        },
        _ => Error::IoError {
            message: format!("Failed to read asset metadata: {}", path.display()),
            source: std::io::Error::other(format!("{e}")),
        },
    })?;

    let size = metadata.size;

    if size > max_size {
        return Err(Error::AssetTooLarge {
            path: path.display().to_string(),
            size,
            max_size,
        });
    }

    Ok(size)
}

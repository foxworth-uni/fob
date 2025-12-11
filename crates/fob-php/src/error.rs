//! Error handling for PHP bindings

use ext_php_rs::exception::PhpException;

/// Convert bundler error to PHP exception with details
pub fn bundler_error_to_php_exception(error: fob_bundler::Error) -> PhpException {
    let details = map_bundler_error(&error);
    PhpException::default(details)
}

/// Map bundler error to a user-friendly message
fn map_bundler_error(error: &fob_bundler::Error) -> String {
    match error {
        fob_bundler::Error::InvalidConfig(msg) => {
            if msg.contains("No entries provided") || msg.contains("No entry") {
                "No entries provided".to_string()
            } else {
                format!("Invalid configuration: {}", msg)
            }
        }
        fob_bundler::Error::InvalidOutputPath(path) => {
            format!("Invalid output path: {}", path)
        }
        fob_bundler::Error::WriteFailure(msg) => {
            format!("Write failure: {}", msg)
        }
        fob_bundler::Error::OutputExists(msg) => {
            format!("Output exists: {}", msg)
        }
        fob_bundler::Error::Bundler(diagnostics) => {
            if diagnostics.is_empty() {
                "Unknown bundler error".to_string()
            } else if diagnostics.len() == 1 {
                format!("Bundler error: {}", diagnostics[0].message)
            } else {
                format!("Multiple bundler errors ({} total)", diagnostics.len())
            }
        }
        fob_bundler::Error::Io(e) => {
            format!("I/O error: {}", e)
        }
        fob_bundler::Error::IoError { message, .. } => message.clone(),
        fob_bundler::Error::AssetNotFound {
            specifier,
            searched_from,
        } => {
            format!(
                "Asset not found: {} (searched from: {})",
                specifier, searched_from
            )
        }
        fob_bundler::Error::AssetSecurityViolation { path, reason } => {
            format!("Asset security violation: {} - {}", path, reason)
        }
        fob_bundler::Error::AssetTooLarge {
            path,
            size,
            max_size,
        } => {
            format!(
                "Asset too large: {} ({} bytes exceeds limit of {} bytes)",
                path, size, max_size
            )
        }
        fob_bundler::Error::Foundation(e) => {
            format!("Foundation error: {}", e)
        }
    }
}

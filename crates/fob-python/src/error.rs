//! Error handling for Python bindings

use pyo3::Bound;
use pyo3::create_exception;
use pyo3::prelude::*;

create_exception!(fob, FobError, pyo3::exceptions::PyRuntimeError);

/// Convert bundler error to Python exception with details
pub fn bundler_error_to_py_err(error: fob_bundler::Error) -> PyErr {
    // Use the same error mapping logic as fob-native
    let details = map_bundler_error(&error);

    // Create a detailed error message
    PyErr::new::<FobError, _>(details.to_string())
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

pub fn register_errors(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    // FobError is automatically registered by create_exception!
    Ok(())
}

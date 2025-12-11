//! Error handling for Ruby bindings

use magnus::{Error, Module, RModule, Ruby};

/// Register the Fob::Error exception class
pub fn register_error_class(ruby: &Ruby, module: RModule) -> Result<(), Error> {
    // Use define_error for exception classes - it takes ExceptionClass as superclass
    let _error_class = module.define_error("Error", ruby.exception_standard_error())?;
    Ok(())
}

/// Convert bundler error to Ruby exception with details
///
/// Note: This returns a closure that captures the error details,
/// which can then be called with a Ruby handle to create the actual error.
pub fn bundler_error_to_ruby_err(error: fob_bundler::Error) -> impl FnOnce(&Ruby) -> Error {
    let details = map_bundler_error(&error);
    move |ruby: &Ruby| Error::new(ruby.exception_runtime_error(), details)
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

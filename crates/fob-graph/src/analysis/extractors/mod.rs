//! Framework script extractors.
//!
//! This module provides extractors for extracting JavaScript/TypeScript from
//! framework-specific file formats.
//!
//! # Usage
//!
//! ```rust,no_run
//! use fob_graph::analysis::extractors::{extract_scripts, ExtractedScript};
//! use std::path::Path;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let path = Path::new("component.js");
//! let content = std::fs::read_to_string(path)?;
//! let scripts = extract_scripts(path, &content)?;
//!
//! for script in scripts {
//!     println!("Found script: {}", script.source_text);
//! }
//! # Ok(()) }
//! ```

mod common;

pub use common::{ExtractedScript, Extractor, ExtractorError, ScriptContext};

// Re-export constants for convenience
pub use common::{MAX_FILE_SIZE, MAX_SCRIPT_TAGS};

use std::path::Path;

/// Extract scripts from a framework file, auto-detecting the framework by extension.
///
/// # Arguments
///
/// * `path` - The file path (used to determine framework by extension)
/// * `content` - The file content
///
/// # Returns
///
/// A vector of extracted scripts, or an error if extraction fails.
///
/// Currently, no framework-specific extractors are supported.
/// This function returns an empty vector for all file types.
///
/// # Example
///
/// ```rust,no_run
/// use fob_graph::analysis::extractors::extract_scripts;
/// use std::path::Path;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let path = Path::new("component.js");
/// let content = "const x = 1;";
///
/// let scripts = extract_scripts(path, content)?;
/// assert_eq!(scripts.len(), 0);
/// # Ok(()) }
/// ```
pub fn extract_scripts<'a>(
    _path: &Path,
    _content: &'a str,
) -> Result<Vec<ExtractedScript<'a>>, ExtractorError> {
    // No framework extractors currently supported
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_scripts_unknown_extension() {
        let path = Path::new("test.js");
        let content = "const x = 1";
        let scripts = extract_scripts(path, content).unwrap();
        assert_eq!(scripts.len(), 0);
    }
}

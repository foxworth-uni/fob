//! Utility functions for Python API

use pyo3::prelude::*;
use pyo3::types::PyString;
use std::path::PathBuf;

/// Convert a Python object to a string path
///
/// Accepts both str and pathlib.Path objects
pub fn py_to_path_string(obj: &Bound<'_, PyAny>) -> PyResult<String> {
    // Try to extract as string first
    if let Ok(s) = obj.extract::<String>() {
        return Ok(s);
    }

    // Try to extract as PathBuf (from pathlib.Path)
    if let Ok(path) = obj.extract::<PathBuf>() {
        return Ok(path.to_string_lossy().to_string());
    }

    // Try to get string representation
    if let Ok(s) = obj.cast::<PyString>() {
        return Ok(s.to_string_lossy().to_string());
    }

    Err(PyErr::new::<crate::error::FobError, _>(format!(
        "Expected str or Path, got {}",
        obj.get_type().name()?
    )))
}

/// Convert a Python object to a list of string paths
pub fn py_to_path_strings(obj: &Bound<'_, PyAny>) -> PyResult<Vec<String>> {
    // Try as list first
    if let Ok(list) = obj.cast::<pyo3::types::PyList>() {
        let mut paths = Vec::new();
        for item in list.iter() {
            paths.push(py_to_path_string(&item)?);
        }
        return Ok(paths);
    }

    // Try as single string/path
    Ok(vec![py_to_path_string(obj)?])
}

/// Normalize a string (lowercase, trim)
pub fn normalize_string(s: &str) -> String {
    s.trim().to_lowercase()
}

/// Parse format string with normalization
pub fn parse_format_normalized(s: &str) -> Option<crate::types::OutputFormat> {
    crate::types::OutputFormat::from_str(&normalize_string(s))
}

/// Parse platform string with normalization
pub fn parse_platform_normalized(s: &str) -> Option<String> {
    let normalized = normalize_string(s);
    match normalized.as_str() {
        "browser" | "web" => Some("browser".to_string()),
        "node" | "nodejs" => Some("node".to_string()),
        _ => None,
    }
}

/// Parse entry mode string with normalization
pub fn parse_entry_mode_normalized(s: &str) -> Option<crate::api::primitives::EntryMode> {
    crate::api::primitives::EntryMode::from_str(&normalize_string(s))
}

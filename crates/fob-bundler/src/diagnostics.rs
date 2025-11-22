//! Diagnostic extraction from Rolldown errors.
//!
//! This module provides structured extraction of diagnostic information from
//! Rolldown's error types, creating an abstraction layer that insulates us
//! from upstream API changes.

use serde::{Deserialize, Serialize};

/// Extracted diagnostic information from Rolldown.
///
/// This struct contains all the information we need from Rolldown diagnostics
/// in a cloneable, serializable format that's stable across Rolldown versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedDiagnostic {
    pub kind: DiagnosticKind,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub help: Option<String>,
}

/// Diagnostic kind (mirrors Rolldown's EventKind).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticKind {
    MissingExport,
    ParseError,
    CircularDependency,
    UnresolvedEntry,
    UnresolvedImport,
    InvalidOption,
    Plugin,
    Transform,
    Other(String),
}

/// Diagnostic severity level.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

/// Extract diagnostics from Rolldown error types.
///
/// This function attempts to extract structured information from Rolldown's
/// error types. Since Rolldown's API may change, we use public methods
/// and fall back to parsing formatted messages when necessary.
pub fn extract_from_rolldown_error(error: &dyn std::fmt::Debug) -> Vec<ExtractedDiagnostic> {
    // Convert error to string for analysis
    let error_str = format!("{error:?}");

    // Try to extract structured information
    // For now, we'll parse the debug string format
    // TODO: Use Rolldown's public API when available

    // Check if this looks like a batched error (multiple diagnostics)
    if error_str.contains("BatchedBuildDiagnostic") || error_str.contains("diagnostics") {
        // Try to extract multiple diagnostics
        return extract_multiple_from_string(&error_str);
    }

    // Single diagnostic
    vec![extract_single_from_string(&error_str)]
}

/// Extract a single diagnostic from a formatted error string.
fn extract_single_from_string(error_str: &str) -> ExtractedDiagnostic {
    // Determine kind from error message
    let kind = if error_str.contains("MissingExport") {
        DiagnosticKind::MissingExport
    } else if error_str.contains("Parse error")
        || error_str.contains("Syntax")
        || error_str.contains("Expected")
    {
        DiagnosticKind::ParseError
    } else if error_str.contains("Circular") || error_str.contains("cycle") {
        DiagnosticKind::CircularDependency
    } else if error_str.contains("UnresolvedEntry") || error_str.contains("entry") {
        DiagnosticKind::UnresolvedEntry
    } else if error_str.contains("UnresolvedImport") || error_str.contains("Cannot resolve") {
        DiagnosticKind::UnresolvedImport
    } else if error_str.contains("Plugin") {
        DiagnosticKind::Plugin
    } else if error_str.contains("Transform") || error_str.contains("transform") {
        DiagnosticKind::Transform
    } else {
        DiagnosticKind::Other(error_str.to_string())
    };

    // Extract severity (default to error)
    let severity = if error_str.contains("warning") || error_str.contains("Warning") {
        DiagnosticSeverity::Warning
    } else {
        DiagnosticSeverity::Error
    };

    // Extract file path
    let file = extract_file_path(error_str);

    // Extract line number
    let line = extract_line_number(error_str);

    // Extract column number
    let column = extract_column_number(error_str);

    // Extract help text if available
    let help = extract_help_text(error_str);

    ExtractedDiagnostic {
        kind,
        severity,
        message: error_str.to_string(),
        file,
        line,
        column,
        help,
    }
}

/// Extract multiple diagnostics from a batched error string.
fn extract_multiple_from_string(error_str: &str) -> Vec<ExtractedDiagnostic> {
    // For now, split by common separators and extract each
    // This is a fallback - ideally we'd use Rolldown's API
    let parts: Vec<&str> = error_str
        .split("BatchedBuildDiagnostic")
        .filter(|s| !s.trim().is_empty())
        .collect();

    if parts.len() > 1 {
        parts
            .iter()
            .map(|part| extract_single_from_string(part))
            .collect()
    } else {
        vec![extract_single_from_string(error_str)]
    }
}

/// Extract file path from error message.
fn extract_file_path(text: &str) -> Option<String> {
    // Look for file paths (typically contain .js, .ts, .jsx, .tsx)
    for ext in &[".js", ".ts", ".jsx", ".tsx", ".mjs", ".cjs"] {
        if let Some(pos) = text.find(ext) {
            // Backtrack to find the start of the path
            let before = &text[..=pos + ext.len()];
            // Look for common path indicators
            for indicator in &["in ", "at ", "file: ", "path: ", "\"", "'"] {
                if let Some(start) = before.rfind(indicator) {
                    let path_start = start + indicator.len();
                    let path_str = &before[path_start..];
                    // Find end of path (space, newline, quote, comma)
                    if let Some(end) = path_str.find([' ', '\n', '"', '\'', ',']) {
                        return Some(path_str[..end].trim().to_string());
                    }
                    return Some(path_str.trim().to_string());
                }
            }
        }
    }
    None
}

/// Extract line number from error message.
fn extract_line_number(text: &str) -> Option<u32> {
    // Look for patterns like "line 5", ":5:", "line:5", etc.
    for pattern in &["line ", ":"] {
        if let Some(pos) = text.find(pattern) {
            let after = &text[pos + pattern.len()..];
            // Find the number
            let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !num_str.is_empty() {
                if let Ok(num) = num_str.parse::<u32>() {
                    return Some(num);
                }
            }
        }
    }
    None
}

/// Extract column number from error message.
fn extract_column_number(text: &str) -> Option<u32> {
    // Look for patterns like "column 10", ":5:10", "col:10", etc.
    for pattern in &["column ", "col ", ":"] {
        if let Some(pos) = text.find(pattern) {
            let after = &text[pos + pattern.len()..];
            // Skip if this is actually a line number pattern
            if *pattern == ":" && after.starts_with(|c: char| c.is_ascii_digit()) {
                // This might be line:column format, skip to next colon
                if let Some(colon_pos) = after[1..].find(':') {
                    let after_colon = &after[colon_pos + 2..];
                    let num_str: String = after_colon
                        .chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    if !num_str.is_empty() {
                        if let Ok(num) = num_str.parse::<u32>() {
                            return Some(num);
                        }
                    }
                }
            } else {
                let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
                if !num_str.is_empty() {
                    if let Ok(num) = num_str.parse::<u32>() {
                        return Some(num);
                    }
                }
            }
        }
    }
    None
}

/// Extract help text from error message.
fn extract_help_text(text: &str) -> Option<String> {
    // Look for help indicators
    for indicator in &[
        "help: ",
        "Help: ",
        "hint: ",
        "Hint: ",
        "suggestion: ",
        "Suggestion: ",
    ] {
        if let Some(pos) = text.find(indicator) {
            let after = &text[pos + indicator.len()..];
            // Take until newline or end
            let help_str: String = after.lines().next().unwrap_or("").trim().to_string();
            if !help_str.is_empty() {
                return Some(help_str);
            }
        }
    }
    None
}

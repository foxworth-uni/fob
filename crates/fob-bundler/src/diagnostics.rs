//! Diagnostic extraction from Rolldown errors.
//!
//! This module provides structured extraction of diagnostic information from
//! Rolldown's error types, creating an abstraction layer that insulates us
//! from upstream API changes.

mod miette;

pub use miette::{
    DiagnosticError, calculate_enhanced_span, calculate_span_length, clear_source_cache,
    line_col_to_offset, load_source, to_diagnostic_error,
};

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
    /// Structured context for the diagnostic (if available)
    pub context: Option<DiagnosticContext>,
    /// Error chain (causes) extracted from the error
    #[serde(default)]
    pub error_chain: Vec<String>,
}

/// Structured context for different diagnostic kinds.
///
/// Provides type-safe access to diagnostic-specific information without
/// requiring string parsing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DiagnosticContext {
    /// Context for missing export errors
    MissingExport {
        /// Name of the missing export
        export_name: String,
        /// Module ID that requested the export
        module_id: String,
        /// Available exports from the target module
        available_exports: Vec<String>,
    },
    /// Context for circular dependency errors
    CircularDependency {
        /// Path of modules in the cycle
        cycle_path: Vec<String>,
    },
    /// Context for plugin errors
    Plugin {
        /// Name of the plugin that failed
        plugin_name: String,
    },
    /// Context for transform errors
    Transform {
        /// File path that failed to transform
        file_path: String,
    },
    /// Context for unresolved entry errors
    UnresolvedEntry {
        /// Entry path that couldn't be resolved
        entry_path: String,
    },
    /// Context for unresolved import errors
    UnresolvedImport {
        /// Import specifier that couldn't be resolved
        specifier: String,
        /// File that tried to import
        from_file: String,
    },
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

    // Extract error chain (causes)
    let error_chain = extract_error_chain(error_str);

    // Try to extract structured context based on diagnostic kind
    let context = match &kind {
        DiagnosticKind::MissingExport => extract_missing_export_context(error_str, &help),
        DiagnosticKind::CircularDependency => extract_circular_dependency_context(error_str),
        DiagnosticKind::Plugin => extract_plugin_context(error_str),
        DiagnosticKind::Transform => file.as_ref().map(|f| DiagnosticContext::Transform {
            file_path: f.clone(),
        }),
        DiagnosticKind::UnresolvedEntry => {
            file.as_ref().map(|f| DiagnosticContext::UnresolvedEntry {
                entry_path: f.clone(),
            })
        }
        DiagnosticKind::UnresolvedImport => {
            extract_unresolved_import_context(error_str, file.as_ref())
        }
        _ => None,
    };

    ExtractedDiagnostic {
        kind,
        severity,
        message: error_str.to_string(),
        file,
        line,
        column,
        help,
        context,
        error_chain,
    }
}

/// Extract error chain from error message.
///
/// Looks for patterns like:
/// - "Caused by: ..."
/// - "  - cause 1"
/// - "Error chain:\n  - ..."
fn extract_error_chain(text: &str) -> Vec<String> {
    let mut chain = Vec::new();

    // Look for "Caused by:" pattern (common in anyhow errors)
    for line in text.lines() {
        let trimmed = line.trim();

        // Match "Caused by: message" or "  - Caused by: message"
        if let Some(pos) = trimmed.find("Caused by:") {
            let cause = trimmed[pos + 10..].trim();
            if !cause.is_empty() {
                chain.push(cause.to_string());
            }
        }
        // Match "  - message" pattern (bullet points in error chain)
        else if trimmed.starts_with("- ") && !trimmed.contains("->") {
            let cause = trimmed[2..].trim();
            if !cause.is_empty() && !cause.starts_with("Plugin") {
                chain.push(cause.to_string());
            }
        }
    }

    // Also try to extract from "Error chain:" section
    if let Some(chain_start) = text.find("Error chain:") {
        let chain_section = &text[chain_start + 12..];
        for line in chain_section.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("- ") {
                let cause = trimmed[2..].trim();
                if !cause.is_empty() && !chain.contains(&cause.to_string()) {
                    chain.push(cause.to_string());
                }
            } else if trimmed.is_empty() {
                // End of chain section
                break;
            }
        }
    }

    chain
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

/// Extract context for missing export errors
fn extract_missing_export_context(
    error_str: &str,
    help: &Option<String>,
) -> Option<DiagnosticContext> {
    // Try to extract export name
    let export_name = extract_quoted_string_after(error_str, "export")
        .or_else(|| extract_quoted_string_after(error_str, "MissingExport"))
        .unwrap_or_else(|| "unknown".to_string());

    // Try to extract module ID
    let module_id = extract_file_path(error_str)
        .or_else(|| extract_quoted_string_after(error_str, "module"))
        .unwrap_or_else(|| "unknown".to_string());

    // Try to extract available exports from help text
    let available_exports = help
        .as_ref()
        .and_then(|h| extract_available_exports_list(h))
        .unwrap_or_default();

    Some(DiagnosticContext::MissingExport {
        export_name,
        module_id,
        available_exports,
    })
}

/// Extract context for circular dependency errors
fn extract_circular_dependency_context(error_str: &str) -> Option<DiagnosticContext> {
    // Try to extract cycle path (look for "->" or "→" patterns)
    let cycle_path = if let Some(start) = error_str.find("->") {
        error_str[start..]
            .split("->")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else if let Some(start) = error_str.find("→") {
        error_str[start..]
            .split("→")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        // Fallback: try to extract file paths
        let mut paths = Vec::new();
        if let Some(file) = extract_file_path(error_str) {
            paths.push(file);
        }
        paths
    };

    if !cycle_path.is_empty() {
        Some(DiagnosticContext::CircularDependency { cycle_path })
    } else {
        None
    }
}

/// Extract context for plugin errors
fn extract_plugin_context(error_str: &str) -> Option<DiagnosticContext> {
    // Try to extract plugin name
    let plugin_name = extract_quoted_string_after(error_str, "Plugin")
        .or_else(|| {
            // Look for patterns like "Plugin 'name'"
            if let Some(start) = error_str.find("Plugin") {
                let after = &error_str[start + 6..];
                extract_quoted_string(after)
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    Some(DiagnosticContext::Plugin { plugin_name })
}

/// Extract context for unresolved import errors
fn extract_unresolved_import_context(
    error_str: &str,
    file: Option<&String>,
) -> Option<DiagnosticContext> {
    // Try to extract specifier
    let specifier = extract_quoted_string_after(error_str, "Cannot resolve")
        .or_else(|| extract_quoted_string_after(error_str, "import"))
        .unwrap_or_else(|| "unknown".to_string());

    let from_file = file.cloned().unwrap_or_else(|| "unknown".to_string());

    Some(DiagnosticContext::UnresolvedImport {
        specifier,
        from_file,
    })
}

/// Extract a quoted string after a keyword
fn extract_quoted_string_after(text: &str, keyword: &str) -> Option<String> {
    if let Some(pos) = text.find(keyword) {
        let after = &text[pos + keyword.len()..];
        extract_quoted_string(after)
    } else {
        None
    }
}

/// Extract a quoted string (single, double, or backtick)
fn extract_quoted_string(text: &str) -> Option<String> {
    for quote in &['"', '\'', '`'] {
        if let Some(start) = text.find(*quote) {
            let after = &text[start + 1..];
            if let Some(end) = after.find(*quote) {
                return Some(after[..end].to_string());
            }
        }
    }
    None
}

/// Extract available exports from help text
fn extract_available_exports_list(help: &str) -> Option<Vec<String>> {
    // Look for patterns like "Available: Foo, Bar, Baz" or "Available exports: ..."
    for pattern in &["Available: ", "Available exports: ", "Exports: "] {
        if let Some(pos) = help.find(pattern) {
            let after = &help[pos + pattern.len()..];
            let exports_str = after.lines().next().unwrap_or("").trim();
            if !exports_str.is_empty() {
                let exports: Vec<String> = exports_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !exports.is_empty() {
                    return Some(exports);
                }
            }
        }
    }
    None
}

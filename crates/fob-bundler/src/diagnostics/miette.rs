//! Miette diagnostic conversion for fob-bundler errors.
//!
//! This module provides conversion from fob-bundler diagnostics to miette's
//! diagnostic format for beautiful error reporting.

use crate::diagnostics::{DiagnosticKind, DiagnosticSeverity, ExtractedDiagnostic};
use miette::{Diagnostic, LabeledSpan, Severity, SourceSpan};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{LazyLock, Mutex};

/// Cache for loaded source code files
static SOURCE_CACHE: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// Load source code from a file path
pub fn load_source(file: &str) -> Option<String> {
    let mut cache = SOURCE_CACHE.lock().unwrap();
    
    // Check cache first
    if let Some(source) = cache.get(file) {
        return Some(source.clone());
    }

    // Try to load from file system
    let path = Path::new(file);
    if path.exists() && path.is_file() {
        if let Ok(content) = std::fs::read_to_string(path) {
            cache.insert(file.to_string(), content.clone());
            return Some(content);
        }
    }

    None
}


/// Convert line and column to byte offset
pub fn line_col_to_offset(source: &str, line: u32, column: u32) -> Option<usize> {
    let lines: Vec<&str> = source.lines().collect();
    if line == 0 || line as usize > lines.len() {
        return None;
    }

    let line_idx = (line - 1) as usize;
    let mut offset = 0;

    // Sum up bytes for all lines before the target line
    for i in 0..line_idx {
        offset += lines[i].len() + 1; // +1 for newline
    }

    // Add column offset (handle UTF-8 by using byte position)
    let target_line = lines[line_idx];
    let col_bytes = if column == 0 {
        0
    } else {
        target_line
            .char_indices()
            .nth((column - 1) as usize)
            .map(|(pos, _)| pos)
            .unwrap_or(target_line.len())
    };

    Some(offset + col_bytes)
}

/// Calculate span length from source code
pub fn calculate_span_length(source: &str, offset: usize) -> usize {
    if offset >= source.len() {
        return 1;
    }
    
    // Try to find the end of the current token/identifier
    let remaining = &source[offset..];
    
    // For identifiers, find the end of the word
    if let Some(end) = remaining
        .char_indices()
        .find(|(_, c)| !c.is_alphanumeric() && *c != '_')
        .map(|(pos, _)| pos)
    {
        end.max(1)
    } else {
        // Default to highlighting a single character
        1
    }
}

/// Enhanced span calculation for better error highlighting
pub fn calculate_enhanced_span(source: &str, offset: usize, kind: &DiagnosticKind) -> usize {
    if offset >= source.len() {
        return 1;
    }
    
    let remaining = &source[offset..];
    
    match kind {
        DiagnosticKind::MissingExport => {
            // For missing exports, try to highlight the import statement
            // Look for import/export keywords nearby
            if let Some(import_pos) = remaining.find("import") {
                if import_pos < 50 {
                    // Highlight from import to end of line or semicolon
                    if let Some(end) = remaining[import_pos..].find([';', '\n']) {
                        return (import_pos + end).max(1);
                    }
                }
            }
            calculate_span_length(source, offset)
        }
        DiagnosticKind::ParseError | DiagnosticKind::Transform => {
            // For parse errors, try to highlight the problematic token
            // Look for common syntax error patterns
            if let Some(quote_pos) = remaining.find(['"', '\'', '`']) {
                if quote_pos < 20 {
                    // Find matching quote
                    let quote_char = remaining.as_bytes()[quote_pos] as char;
                    if let Some(end_quote) = remaining[quote_pos + 1..].find(quote_char) {
                        return (quote_pos + end_quote + 2).max(1);
                    }
                }
            }
            calculate_span_length(source, offset)
        }
        _ => calculate_span_length(source, offset),
    }
}

/// Wrapper error type that implements Diagnostic for ExtractedDiagnostic
#[derive(Debug)]
pub struct DiagnosticError {
    diag: ExtractedDiagnostic,
    source_code: Option<(String, String)>, // (file, source)
}

impl std::error::Error for DiagnosticError {}

impl std::fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.diag.message)
    }
}

impl Diagnostic for DiagnosticError {
    fn code(&self) -> Option<Box<dyn std::fmt::Display + '_>> {
        Some(Box::new(format!("{:?}", self.diag.kind)))
    }

    fn severity(&self) -> Option<Severity> {
        Some(match self.diag.severity {
            DiagnosticSeverity::Error => Severity::Error,
            DiagnosticSeverity::Warning => Severity::Warning,
        })
    }

    fn help(&self) -> Option<Box<dyn std::fmt::Display + '_>> {
        self.diag.help.as_ref().map(|h| Box::new(h.clone()) as Box<dyn std::fmt::Display>)
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        // Miette requires returning a reference, but we own the source
        // We'll handle source code display via labels and the file path
        // The actual source code will be loaded by miette's report handler
        None
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        if let (Some(_file), Some(line), Some(column)) = (&self.diag.file, self.diag.line, self.diag.column) {
            if let Some((_, source)) = &self.source_code {
                if let Some(offset) = line_col_to_offset(source, line, column) {
                    let length = calculate_enhanced_span(source, offset, &self.diag.kind);
                    let span = SourceSpan::new(offset.into(), length.into());
                    
                    let label = match &self.diag.kind {
                        DiagnosticKind::MissingExport => "Missing export",
                        DiagnosticKind::ParseError => "Parse error",
                        DiagnosticKind::CircularDependency => "Circular dependency",
                        DiagnosticKind::UnresolvedEntry => "Unresolved entry",
                        DiagnosticKind::UnresolvedImport => "Unresolved import",
                        DiagnosticKind::InvalidOption => "Invalid option",
                        DiagnosticKind::Plugin => "Plugin error",
                        DiagnosticKind::Transform => "Transform error",
                        DiagnosticKind::Other(_) => "Error",
                    };

                    return Some(Box::new(std::iter::once(LabeledSpan::new(
                        Some(label.to_string()),
                        span.offset(),
                        span.len(),
                    ))));
                }
            }
        }
        None
    }
}

/// Convert ExtractedDiagnostic to a DiagnosticError with source code loaded
pub fn to_diagnostic_error(diag: ExtractedDiagnostic) -> DiagnosticError {
    let source_code = diag.file.as_ref()
        .and_then(|file| load_source(file).map(|source| (file.clone(), source)));
    
    DiagnosticError {
        diag,
        source_code,
    }
}


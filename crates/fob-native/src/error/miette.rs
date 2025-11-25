//! Miette diagnostic conversion for fob-native errors.
//!
//! This module provides conversion from fob-native error types to miette diagnostics
//! for beautiful error reporting in CLI contexts.

use crate::error::FobErrorDetails;
use ::miette::{Diagnostic, LabeledSpan, Severity, SourceSpan};
use fob_bundler::diagnostics::{calculate_span_length, line_col_to_offset, load_source};

/// Wrapper to implement Diagnostic for FobErrorDetails
#[derive(Debug)]
pub struct FobErrorDiagnostic {
    error: FobErrorDetails,
}

impl std::error::Error for FobErrorDiagnostic {}

impl std::fmt::Display for FobErrorDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.error {
            FobErrorDetails::MdxSyntax(e) => write!(f, "{}", e.message),
            FobErrorDetails::MissingExport(e) => {
                write!(
                    f,
                    "Missing export '{}' from module '{}'",
                    e.export_name, e.module_id
                )
            }
            FobErrorDetails::Transform(e) => write!(f, "Transform error in {}", e.path),
            FobErrorDetails::CircularDependency(e) => {
                write!(f, "Circular dependency: {}", e.cycle_path.join(" -> "))
            }
            FobErrorDetails::InvalidEntry(e) => write!(f, "Invalid entry: {}", e.path),
            FobErrorDetails::NoEntries(_) => write!(f, "No entry points specified"),
            FobErrorDetails::Plugin(e) => write!(f, "Plugin '{}' error: {}", e.name, e.message),
            FobErrorDetails::Runtime(e) => write!(f, "{}", e.message),
            FobErrorDetails::Validation(e) => write!(f, "{}", e.message),
            FobErrorDetails::Multiple(e) => write!(f, "{}", e.primary_message),
        }
    }
}

impl Diagnostic for FobErrorDiagnostic {
    fn code(&self) -> Option<Box<dyn std::fmt::Display + '_>> {
        Some(Box::new(match &self.error {
            FobErrorDetails::MdxSyntax(_) => "MDX_SYNTAX_ERROR",
            FobErrorDetails::MissingExport(_) => "MISSING_EXPORT",
            FobErrorDetails::Transform(_) => "TRANSFORM_ERROR",
            FobErrorDetails::CircularDependency(_) => "CIRCULAR_DEPENDENCY",
            FobErrorDetails::InvalidEntry(_) => "INVALID_ENTRY",
            FobErrorDetails::NoEntries(_) => "NO_ENTRIES",
            FobErrorDetails::Plugin(_) => "PLUGIN_ERROR",
            FobErrorDetails::Runtime(_) => "RUNTIME_ERROR",
            FobErrorDetails::Validation(_) => "VALIDATION_ERROR",
            FobErrorDetails::Multiple(_) => "MULTIPLE_ERRORS",
        }))
    }

    fn severity(&self) -> Option<Severity> {
        Some(Severity::Error)
    }

    fn help(&self) -> Option<Box<dyn std::fmt::Display + '_>> {
        match &self.error {
            FobErrorDetails::MdxSyntax(e) => e.suggestion.as_ref().map(|s| Box::new(s.clone()) as Box<dyn std::fmt::Display>),
            FobErrorDetails::MissingExport(e) => {
                if !e.available_exports.is_empty() {
                    Some(Box::new(format!(
                        "Available exports: {}\n{}",
                        e.available_exports.join(", "),
                        e.suggestion.as_deref().unwrap_or("")
                    )) as Box<dyn std::fmt::Display>)
                } else {
                    e.suggestion.as_ref().map(|s| Box::new(s.clone()) as Box<dyn std::fmt::Display>)
                }
            }
            FobErrorDetails::CircularDependency(e) => {
                Some(Box::new(format!(
                    "Circular dependency detected:\n{}\n\nHint: Refactor to remove circular imports by extracting shared code into a separate module.",
                    e.cycle_path.join(" -> ")
                )) as Box<dyn std::fmt::Display>)
            }
            FobErrorDetails::InvalidEntry(e) => {
                Some(Box::new(format!(
                    "The entry point '{}' is invalid.\nHint: Check that the file exists and the path is correct.",
                    e.path
                )) as Box<dyn std::fmt::Display>)
            }
            FobErrorDetails::NoEntries(_) => {
                Some(Box::new("At least one entry point is required.\nHint: Specify entry points in your config or use --entry flag.") as Box<dyn std::fmt::Display>)
            }
            FobErrorDetails::Plugin(e) => {
                Some(Box::new(format!(
                    "Plugin '{}' encountered an error.\nHint: Check the plugin configuration and ensure all dependencies are installed.",
                    e.name
                )) as Box<dyn std::fmt::Display>)
            }
            FobErrorDetails::Transform(e) => {
                if let Some(first) = e.diagnostics.first() {
                    first.help.as_ref().map(|h| Box::new(h.clone()) as Box<dyn std::fmt::Display>)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        match &self.error {
            FobErrorDetails::MdxSyntax(e) => {
                if let (Some(file), Some(line), Some(column)) = (&e.file, e.line, e.column) {
                    if let Some(source) = load_source(file) {
                        if let Some(offset) = line_col_to_offset(&source, line, column) {
                            let length = calculate_span_length(&source, offset);
                            let span = SourceSpan::new(offset.into(), length.into());
                            return Some(Box::new(std::iter::once(LabeledSpan::new(
                                Some("MDX syntax error".to_string()),
                                span.offset(),
                                span.len(),
                            ))));
                        }
                    }
                }
                None
            }
            FobErrorDetails::Transform(e) => {
                if let Some(first) = e.diagnostics.first() {
                    if let Some(source) = load_source(&e.path) {
                        if let Some(offset) = line_col_to_offset(&source, first.line, first.column)
                        {
                            let length = calculate_span_length(&source, offset);
                            let span = SourceSpan::new(offset.into(), length.into());
                            return Some(Box::new(std::iter::once(LabeledSpan::new(
                                Some(first.message.clone()),
                                span.offset(),
                                span.len(),
                            ))));
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn related(&self) -> Option<Box<dyn Iterator<Item = &dyn Diagnostic> + '_>> {
        match &self.error {
            FobErrorDetails::Multiple(_multiple) => {
                // Convert related errors to diagnostics
                // We can't return references to local values, so return None
                // The primary message will contain the summary
                None
            }
            _ => None,
        }
    }
}

/// Convert FobErrorDetails to a miette diagnostic
pub fn to_miette_diagnostic(error: FobErrorDetails) -> FobErrorDiagnostic {
    FobErrorDiagnostic { error }
}

//! Production-grade error handling for WASM bindings
//!
//! This module provides structured, type-safe error handling that preserves
//! error context across the WASM boundary and provides JavaScript-friendly
//! error objects with full TypeScript support.

use fob_mdx::MdxError;
use serde::{Deserialize, Serialize};
use std::fmt;
use wasm_bindgen::prelude::*;

/// Error categories specific to WASM context
///
/// These categories help JavaScript consumers handle different error
/// types appropriately. The `kind` field enables type discrimination
/// in TypeScript through tagged union types.
///
/// Note: Variant names intentionally include "Error" suffix for serde serialization
/// to produce JSON like `{"kind": "validationError", ...}` as required by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[allow(clippy::enum_variant_names)]
pub enum WasmError {
    /// Input validation failed (size limits, invalid characters, etc.)
    #[serde(rename_all = "camelCase")]
    ValidationError {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<String>,
    },

    /// MDX compilation failed (syntax errors, invalid JSX, etc.)
    #[serde(rename_all = "camelCase")]
    CompilationError {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        location: Option<ErrorLocation>,
        #[serde(skip_serializing_if = "Option::is_none")]
        context: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        suggestion: Option<String>,
    },

    /// Failed to serialize result to JavaScript
    #[serde(rename_all = "camelCase")]
    SerializationError {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<String>,
    },

    /// Internal/unexpected error
    #[serde(rename_all = "camelCase")]
    InternalError {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<String>,
    },
}

/// Source code location information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorLocation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
}

impl WasmError {
    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
            details: None,
        }
    }

    /// Create a validation error with details
    pub fn validation_with_details(message: impl Into<String>, details: impl Into<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
            details: Some(details.into()),
        }
    }

    /// Create a compilation error from an MdxError
    pub fn from_mdx_error(err: MdxError) -> Self {
        let location = if err.file.is_some() || err.line.is_some() || err.column.is_some() {
            Some(ErrorLocation {
                file: err.file,
                line: err.line,
                column: err.column,
            })
        } else {
            None
        };

        Self::CompilationError {
            message: err.message,
            location,
            context: err.context,
            suggestion: err.suggestion,
        }
    }

    /// Create a serialization error
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::SerializationError {
            message: message.into(),
            details: None,
        }
    }

    /// Create a serialization error with details
    pub fn serialization_with_details(
        message: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self::SerializationError {
            message: message.into(),
            details: Some(details.into()),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
            details: None,
        }
    }

    /// Get the error kind as a string (for logging/debugging)
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::ValidationError { .. } => "ValidationError",
            Self::CompilationError { .. } => "CompilationError",
            Self::SerializationError { .. } => "SerializationError",
            Self::InternalError { .. } => "InternalError",
        }
    }

    /// Get the primary error message
    pub fn message(&self) -> &str {
        match self {
            Self::ValidationError { message, .. }
            | Self::CompilationError { message, .. }
            | Self::SerializationError { message, .. }
            | Self::InternalError { message, .. } => message,
        }
    }
}

impl fmt::Display for WasmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationError { message, details } => {
                write!(f, "Validation Error: {}", message)?;
                if let Some(d) = details {
                    write!(f, "\nDetails: {}", d)?;
                }
            }
            Self::CompilationError {
                message,
                location,
                context,
                suggestion,
            } => {
                write!(f, "Compilation Error: {}", message)?;
                if let Some(loc) = location {
                    if let Some(file) = &loc.file {
                        write!(f, "\n  in {}", file)?;
                    }
                    if let (Some(line), Some(col)) = (loc.line, loc.column) {
                        write!(f, "\n  at line {}, column {}", line, col)?;
                    }
                }
                if let Some(ctx) = context {
                    write!(f, "\n\n{}", ctx)?;
                }
                if let Some(sug) = suggestion {
                    write!(f, "\nSuggestion: {}", sug)?;
                }
            }
            Self::SerializationError { message, details } => {
                write!(f, "Serialization Error: {}", message)?;
                if let Some(d) = details {
                    write!(f, "\nDetails: {}", d)?;
                }
            }
            Self::InternalError { message, details } => {
                write!(f, "Internal Error: {}", message)?;
                if let Some(d) = details {
                    write!(f, "\nDetails: {}", d)?;
                }
            }
        }
        Ok(())
    }
}

impl std::error::Error for WasmError {}

// Convert Box<MdxError> to WasmError
impl From<Box<MdxError>> for WasmError {
    fn from(err: Box<MdxError>) -> Self {
        Self::from_mdx_error(*err)
    }
}

// Convert MdxError to WasmError
impl From<MdxError> for WasmError {
    fn from(err: MdxError) -> Self {
        Self::from_mdx_error(err)
    }
}

// Convert WasmError to JsValue for WASM boundary crossing
impl From<WasmError> for JsValue {
    fn from(err: WasmError) -> Self {
        // Serialize the error to a JS object
        match serde_wasm_bindgen::to_value(&err) {
            Ok(js_value) => js_value,
            Err(serialization_err) => {
                // Fallback: if serialization fails, return a simple string error
                // This should never happen with our well-defined types, but safety first
                JsValue::from_str(&format!(
                    "Error serialization failed: {} (original error: {})",
                    serialization_err, err
                ))
            }
        }
    }
}

/// Validate input before processing
///
/// Checks for common issues that should be caught early:
/// - Size limits (prevent DoS)
/// - Null bytes (can cause issues in some parsers)
pub fn validate_input(source: &str, max_size: usize) -> Result<(), Box<WasmError>> {
    // Check size limit (default 10MB for WASM environments)
    if source.len() > max_size {
        return Err(Box::new(WasmError::validation_with_details(
            "Input size exceeds maximum allowed",
            format!(
                "Input is {} bytes, maximum is {} bytes ({}MB)",
                source.len(),
                max_size,
                max_size / 1_000_000
            ),
        )));
    }

    // Check for null bytes (can cause issues in C-based parsers)
    if source.contains('\0') {
        return Err(Box::new(WasmError::validation(
            "Input contains null bytes which are not allowed in MDX",
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_creation() {
        let err = WasmError::validation("Invalid input");
        assert_eq!(err.message(), "Invalid input");
        assert_eq!(err.kind_str(), "ValidationError");
    }

    #[test]
    fn test_validation_error_with_details() {
        let err = WasmError::validation_with_details("Too large", "10MB limit exceeded");
        match err {
            WasmError::ValidationError { details, .. } => {
                assert_eq!(details, Some("10MB limit exceeded".to_string()));
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_compilation_error_from_mdx_error() {
        let mdx_err = MdxError::new("Parse failed")
            .with_file("test.mdx")
            .with_location(10, 5)
            .with_suggestion("Check syntax");

        let wasm_err = WasmError::from_mdx_error(mdx_err);

        match wasm_err {
            WasmError::CompilationError {
                message,
                location,
                suggestion,
                ..
            } => {
                assert_eq!(message, "Parse failed");
                assert!(location.is_some());
                let loc = location.unwrap();
                assert_eq!(loc.file, Some("test.mdx".to_string()));
                assert_eq!(loc.line, Some(10));
                assert_eq!(loc.column, Some(5));
                assert_eq!(suggestion, Some("Check syntax".to_string()));
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_serialization_error() {
        let err = WasmError::serialization("Failed to convert");
        assert_eq!(err.kind_str(), "SerializationError");
        assert_eq!(err.message(), "Failed to convert");
    }

    #[test]
    fn test_internal_error() {
        let err = WasmError::internal("Unexpected failure");
        assert_eq!(err.kind_str(), "InternalError");
    }

    #[test]
    fn test_error_display() {
        let err = WasmError::CompilationError {
            message: "Syntax error".to_string(),
            location: Some(ErrorLocation {
                file: Some("test.mdx".to_string()),
                line: Some(5),
                column: Some(10),
            }),
            context: Some("  5 | import { foo } fro './bar'".to_string()),
            suggestion: Some("Did you mean 'from'?".to_string()),
        };

        let display = format!("{}", err);
        assert!(display.contains("Compilation Error"));
        assert!(display.contains("test.mdx"));
        assert!(display.contains("line 5, column 10"));
        assert!(display.contains("Did you mean 'from'?"));
    }

    #[test]
    fn test_validate_input_success() {
        let result = validate_input("# Hello World", 10_000_000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_input_size_limit() {
        let large_input = "x".repeat(11_000_000);
        let result = validate_input(&large_input, 10_000_000);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind_str(), "ValidationError");
        assert!(err.message().contains("size exceeds maximum"));
    }

    #[test]
    fn test_validate_input_null_bytes() {
        let input = "Hello\0World";
        let result = validate_input(input, 10_000_000);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message().contains("null bytes"));
    }

    #[test]
    #[cfg(target_family = "wasm")]
    fn test_error_serialization() {
        let err = WasmError::validation("Test error");
        let js_value: JsValue = err.into();
        // Should be a valid JS object, not undefined
        assert!(!js_value.is_undefined());
    }

    #[test]
    fn test_from_boxed_mdx_error() {
        let mdx_err = Box::new(MdxError::new("Boxed error"));
        let wasm_err: WasmError = mdx_err.into();
        assert_eq!(wasm_err.message(), "Boxed error");
    }

    #[test]
    fn test_location_without_file() {
        let mdx_err = MdxError::new("Error").with_location(5, 10);
        let wasm_err = WasmError::from_mdx_error(mdx_err);

        match wasm_err {
            WasmError::CompilationError { location, .. } => {
                let loc = location.unwrap();
                assert_eq!(loc.file, None);
                assert_eq!(loc.line, Some(5));
                assert_eq!(loc.column, Some(10));
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_error_without_location() {
        let mdx_err = MdxError::new("Simple error");
        let wasm_err = WasmError::from_mdx_error(mdx_err);

        match wasm_err {
            WasmError::CompilationError { location, .. } => {
                assert!(location.is_none());
            }
            _ => panic!("Wrong error type"),
        }
    }
}

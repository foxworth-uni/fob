//! Error types for JavaScript code generation

use miette::Diagnostic;
use thiserror::Error;

/// Errors that can occur during JavaScript code generation
#[derive(Error, Debug, Diagnostic)]
pub enum GenError {
    /// Invalid identifier name
    #[error("Invalid identifier: '{identifier}'{}", suggestion.as_ref().map(|s| format!(" - {}", s)).unwrap_or_default())]
    #[diagnostic(code(fob::gen::invalid_identifier))]
    InvalidIdentifier {
        identifier: String,
        suggestion: Option<String>,
    },

    /// Code generation failed
    #[error("Code generation failed: {context}{}", reason.as_ref().map(|r| format!(" - {}", r)).unwrap_or_default())]
    #[diagnostic(code(fob::gen::codegen_failed))]
    CodegenFailed {
        context: String,
        reason: Option<String>,
    },

    /// Invalid AST structure
    #[error("Invalid AST structure: {node_type}{}", details.as_ref().map(|d| format!(" - {}", d)).unwrap_or_default())]
    #[diagnostic(code(fob::gen::invalid_ast))]
    InvalidAst {
        node_type: String,
        details: Option<String>,
    },
}

impl GenError {
    /// Create an InvalidIdentifier error
    pub fn invalid_identifier(identifier: impl Into<String>) -> Self {
        Self::InvalidIdentifier {
            identifier: identifier.into(),
            suggestion: None,
        }
    }

    /// Create a CodegenFailed error
    pub fn codegen_failed(context: impl Into<String>) -> Self {
        Self::CodegenFailed {
            context: context.into(),
            reason: None,
        }
    }

    /// Create a CodegenFailed error with reason
    pub fn codegen_failed_with_reason(
        context: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::CodegenFailed {
            context: context.into(),
            reason: Some(reason.into()),
        }
    }

    /// Create an InvalidAst error
    pub fn invalid_ast(node_type: impl Into<String>) -> Self {
        Self::InvalidAst {
            node_type: node_type.into(),
            details: None,
        }
    }
}

/// Result type for code generation operations
pub type Result<T> = std::result::Result<T, GenError>;

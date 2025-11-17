//! Error types for JavaScript code generation

use thiserror::Error;

/// Errors that can occur during JavaScript code generation
#[derive(Error, Debug)]
pub enum GenError {
    /// Invalid identifier name
    #[error("Invalid identifier: {0}")]
    InvalidIdentifier(String),

    /// Code generation failed
    #[error("Codegen failed: {0}")]
    CodegenFailed(String),

    /// Invalid AST structure
    #[error("Invalid AST structure: {0}")]
    InvalidAst(String),
}

/// Result type for code generation operations
pub type Result<T> = std::result::Result<T, GenError>;

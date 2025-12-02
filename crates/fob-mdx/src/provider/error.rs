//! Provider error types

use thiserror::Error;

/// Errors that can occur during provider operations
#[derive(Debug, Error)]
pub enum ProviderError {
    /// Provider not found in registry
    #[error("Unknown provider: {0}")]
    UnknownProvider(String),

    /// Provider method failed during resolution
    #[error("Provider '{provider}' method '{method}' failed: {message}")]
    Resolution {
        provider: String,
        method: String,
        message: String,
    },

    /// Required argument was not provided
    #[error("Missing required argument at index {index}")]
    MissingArgument { index: usize },

    /// Argument has wrong type
    #[error("Invalid argument type at index {index}: expected {expected}")]
    InvalidArgumentType { index: usize, expected: String },

    /// Field not found in provider response
    #[error("Field '{field}' not found in response")]
    FieldNotFound { field: String },

    /// Network error during data fetching
    #[error("Network error: {0}")]
    Network(String),

    /// Parse error when processing response
    #[error("Parse error: {0}")]
    Parse(String),
}

impl ProviderError {
    /// Create a resolution error with the given details
    pub fn resolution(provider: impl Into<String>, method: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Resolution {
            provider: provider.into(),
            method: method.into(),
            message: message.into(),
        }
    }

    /// Create a network error
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network(message.into())
    }

    /// Create a parse error
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse(message.into())
    }
}

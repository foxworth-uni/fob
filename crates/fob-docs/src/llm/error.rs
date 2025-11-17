//! Error types for LLM-enhanced documentation.

use std::fmt;

/// Result type for LLM operations.
pub type Result<T> = std::result::Result<T, LlmError>;

/// Errors that can occur during LLM enhancement.
#[derive(Debug)]
pub enum LlmError {
    /// Ollama service is not running or not accessible.
    OllamaNotRunning {
        url: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Requested model is not available on the Ollama instance.
    ModelNotFound {
        model: String,
        available_models: Vec<String>,
    },

    /// LLM generation failed.
    GenerationFailed {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// LLM response could not be parsed.
    InvalidResponse {
        message: String,
        raw_response: String,
    },

    /// Cache operation failed.
    CacheError {
        operation: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Request timeout.
    Timeout {
        seconds: u64,
    },

    /// I/O error.
    Io(std::io::Error),

    /// JSON serialization/deserialization error.
    Json(serde_json::Error),
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmError::OllamaNotRunning { url, .. } => {
                write!(
                    f,
                    "Cannot connect to Ollama at {url}. Is Ollama running?\n\
                     Try: ollama serve"
                )
            }
            LlmError::ModelNotFound {
                model,
                available_models,
            } => {
                write!(
                    f,
                    "Model '{model}' not found. Available models: {}\n\
                     To download: ollama pull {model}",
                    if available_models.is_empty() {
                        "none (install a model first)".to_string()
                    } else {
                        available_models.join(", ")
                    }
                )
            }
            LlmError::GenerationFailed { message, .. } => {
                write!(f, "LLM generation failed: {message}")
            }
            LlmError::InvalidResponse {
                message,
                raw_response,
            } => {
                let preview = if raw_response.len() > 100 {
                    format!("{}...", &raw_response[..100])
                } else {
                    raw_response.clone()
                };
                write!(
                    f,
                    "Invalid LLM response: {message}\nResponse preview: {preview}"
                )
            }
            LlmError::CacheError { operation, .. } => {
                write!(f, "Cache operation '{operation}' failed")
            }
            LlmError::Timeout { seconds } => {
                write!(f, "LLM request timed out after {seconds}s")
            }
            LlmError::Io(err) => write!(f, "I/O error: {err}"),
            LlmError::Json(err) => write!(f, "JSON error: {err}"),
        }
    }
}

impl std::error::Error for LlmError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LlmError::OllamaNotRunning { source, .. } => Some(&**source as &dyn std::error::Error),
            LlmError::GenerationFailed {
                source: Some(source),
                ..
            } => Some(&**source as &dyn std::error::Error),
            LlmError::CacheError { source, .. } => Some(&**source as &dyn std::error::Error),
            LlmError::Io(err) => Some(err),
            LlmError::Json(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for LlmError {
    fn from(err: std::io::Error) -> Self {
        LlmError::Io(err)
    }
}

impl From<serde_json::Error> for LlmError {
    fn from(err: serde_json::Error) -> Self {
        LlmError::Json(err)
    }
}

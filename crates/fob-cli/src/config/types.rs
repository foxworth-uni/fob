use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Output format for bundled code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Esm,
    Cjs,
    Iife,
}

/// Documentation output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum DocsFormat {
    #[serde(rename = "md")]
    Markdown,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "both")]
    Both,
}

/// LLM configuration for documentation enhancement.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DocsLlmConfig {
    /// Model name to use (e.g., "llama3.2:3b", "codellama:7b")
    #[serde(default = "crate::config::defaults::default_llm_model")]
    pub model: String,

    /// Enhancement mode ("missing", "incomplete", "all")
    #[serde(default = "crate::config::defaults::default_llm_mode")]
    pub mode: String,

    /// Enable caching of LLM responses
    #[serde(default = "crate::config::defaults::default_llm_cache")]
    pub cache: bool,

    /// Custom Ollama server URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl Default for DocsLlmConfig {
    fn default() -> Self {
        Self {
            model: crate::config::defaults::default_llm_model(),
            mode: crate::config::defaults::default_llm_mode(),
            cache: crate::config::defaults::default_llm_cache(),
            url: None,
        }
    }
}

/// Source map generation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SourceMapMode {
    Inline,
    External,
    Hidden,
}

/// Target platform environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Browser,
    Node,
}

// Re-export EsTarget from cli module to avoid duplicate definitions
pub use crate::cli::EsTarget;


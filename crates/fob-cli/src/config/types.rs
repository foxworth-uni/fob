use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Output format for bundled code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Esm,
    Cjs,
    Iife,
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

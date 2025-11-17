use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::bundle::helpers::{default_tailwind_output, default_true};

/// CSS processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssOptions {
    /// Enable CSS processing
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Enable CSS modules (scoped class names)
    #[serde(default)]
    pub modules: bool,

    /// Tailwind CSS integration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tailwind: Option<TailwindOptions>,

    /// PostCSS configuration path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postcss_config: Option<PathBuf>,
}

impl Default for CssOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            modules: false,
            tailwind: None,
            postcss_config: None,
        }
    }
}

/// Tailwind CSS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TailwindOptions {
    /// Enable Tailwind CSS processing
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Path to Tailwind config file (default: tailwind.config.js)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<PathBuf>,

    /// Input CSS file containing @tailwind directives
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<PathBuf>,

    /// Output CSS file path (relative to output_dir)
    #[serde(default = "default_tailwind_output")]
    pub output: String,

    /// Content paths for Tailwind (overrides config file)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<String>>,

    /// Watch mode: run Tailwind in watch mode during dev
    #[serde(default = "default_true")]
    pub watch: bool,
}

impl Default for TailwindOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            config: None,
            input: None,
            output: "styles.css".to_string(),
            content: None,
            watch: true,
        }
    }
}


use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::bundle::helpers::{default_html_filename, default_lang};
use crate::bundle::types::HtmlTemplateType;

/// HTML generation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlOptions {
    /// Path to custom HTML template (Jinja2 format)
    /// If not provided, uses built-in template based on template type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<PathBuf>,

    /// Template type to use when no custom template is specified
    #[serde(default)]
    pub template_type: HtmlTemplateType,

    /// Output filename for generated HTML (default: "index.html")
    #[serde(default = "default_html_filename")]
    pub filename: String,

    /// Page title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Meta description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Meta keywords
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<String>,

    /// Language attribute for <html> tag (default: "en")
    #[serde(default = "default_lang")]
    pub lang: String,

    /// Path to favicon
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,

    /// Custom body HTML content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// Custom head HTML content (injected before closing </head>)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<String>,

    /// Additional template variables (forwarded to template context)
    #[serde(default)]
    pub variables: HashMap<String, Value>,
}

impl Default for HtmlOptions {
    fn default() -> Self {
        Self {
            template: None,
            template_type: HtmlTemplateType::default(),
            filename: "index.html".to_string(),
            title: None,
            description: None,
            keywords: None,
            lang: "en".to_string(),
            favicon: None,
            body: None,
            head: None,
            variables: HashMap::new(),
        }
    }
}

//! Prop definition types for MDX data providers
//!
//! These types represent parsed prop expressions from MDX frontmatter.
//! Frameworks like gumbo use these to resolve data at build/request/client time.

use serde::{Deserialize, Serialize};

/// A parsed prop definition from frontmatter
///
/// Represents expressions like: `github.repo("owner/name").stargazers_count @refresh=60s`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropDefinition {
    /// Variable name in MDX scope (e.g., "stars")
    pub name: String,

    /// Provider identifier (e.g., "github", "notion")
    pub provider: String,

    /// Method to call on provider (e.g., "repo", "database")
    pub method: String,

    /// Arguments to the method
    pub args: Vec<PropArg>,

    /// Field path to extract (e.g., ["stargazers_count"])
    pub fields: Vec<String>,

    /// Options like refresh interval
    pub options: PropOptions,

    /// Original raw expression for debugging
    pub raw: String,
}

/// Argument types for prop methods
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropArg {
    /// String argument
    String(String),
    /// Numeric argument
    Number(f64),
}

impl From<String> for PropArg {
    fn from(s: String) -> Self {
        PropArg::String(s)
    }
}

impl From<&str> for PropArg {
    fn from(s: &str) -> Self {
        PropArg::String(s.to_string())
    }
}

impl From<f64> for PropArg {
    fn from(n: f64) -> Self {
        PropArg::Number(n)
    }
}

/// Options that modify prop behavior
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PropOptions {
    /// Refresh interval (e.g., "60s", "1h")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh: Option<String>,

    /// Run on client only
    #[serde(default, skip_serializing_if = "is_false")]
    pub client: bool,

    /// Run on server only
    #[serde(default, skip_serializing_if = "is_false")]
    pub server: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prop_definition_serialization() {
        let prop = PropDefinition {
            name: "stars".to_string(),
            provider: "github".to_string(),
            method: "repo".to_string(),
            args: vec![PropArg::String("owner/repo".to_string())],
            fields: vec!["stargazers_count".to_string()],
            options: PropOptions {
                refresh: Some("60s".to_string()),
                client: false,
                server: false,
            },
            raw: "github.repo(\"owner/repo\").stargazers_count @refresh=60s".to_string(),
        };

        let json = serde_json::to_string(&prop).unwrap();
        assert!(json.contains("\"name\":\"stars\""));
        assert!(json.contains("\"provider\":\"github\""));

        let parsed: PropDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, prop);
    }

    #[test]
    fn test_prop_options_default() {
        let opts = PropOptions::default();
        assert!(opts.refresh.is_none());
        assert!(!opts.client);
        assert!(!opts.server);
    }
}

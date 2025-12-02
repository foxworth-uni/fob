//! Frontmatter data structures

use super::props::PropDefinition;
use serde_json::Value as JsonValue;

/// Frontmatter data extracted from MDX documents
///
/// Frontmatter can be in YAML or TOML format and is parsed during
/// MDX compilation for build-time access. This avoids runtime parsing overhead.
#[derive(Debug, Clone, PartialEq)]
pub struct FrontmatterData {
    /// The format of the original frontmatter
    pub format: FrontmatterFormat,
    /// Parsed frontmatter as JSON value for easy serialization
    pub data: JsonValue,
    /// Raw source text (for debugging/error messages)
    pub raw: String,
    /// Parsed prop definitions from `props:` section
    pub props: Vec<PropDefinition>,
}

/// Format of the frontmatter block
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontmatterFormat {
    /// YAML format (---)
    Yaml,
    /// TOML format (+++)
    Toml,
}

impl FrontmatterData {
    /// Create new frontmatter data from parsed JSON value
    pub fn new(format: FrontmatterFormat, data: JsonValue, raw: String) -> Self {
        Self {
            format,
            data,
            raw,
            props: Vec::new(),
        }
    }

    /// Create frontmatter with parsed props
    pub fn with_props(mut self, props: Vec<PropDefinition>) -> Self {
        self.props = props;
        self
    }

    /// Get prop names for MDXContent signature
    pub fn prop_names(&self) -> Vec<&str> {
        self.props.iter().map(|p| p.name.as_str()).collect()
    }

    /// Check if frontmatter is empty
    pub fn is_empty(&self) -> bool {
        matches!(&self.data, JsonValue::Object(map) if map.is_empty())
            || matches!(&self.data, JsonValue::Null)
    }
}

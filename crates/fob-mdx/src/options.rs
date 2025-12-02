//! MDX compilation options

use crate::OutputFormat;
use crate::frontmatter::FrontmatterData;
use crate::plugins::MdxPlugin;

/// Configuration options for MDX processing
pub struct MdxOptions {
    /// Plugins for AST and JSX transformation
    pub plugins: Vec<Box<dyn MdxPlugin>>,
    /// JSX runtime import path (default: "react/jsx-runtime")
    pub jsx_runtime: String,
    /// Output format (Program or FunctionBody)
    pub output_format: OutputFormat,
    /// Pre-extracted frontmatter (passed from compile() to avoid double extraction)
    pub frontmatter: Option<FrontmatterData>,
}

impl Default for MdxOptions {
    fn default() -> Self {
        Self {
            plugins: Vec::new(),
            jsx_runtime: "react/jsx-runtime".to_string(),
            output_format: OutputFormat::default(),
            frontmatter: None,
        }
    }
}

impl MdxOptions {
    /// Create new MdxOptions with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a plugin to the options
    pub fn with_plugin(mut self, plugin: Box<dyn MdxPlugin>) -> Self {
        self.plugins.push(plugin);
        self
    }
}

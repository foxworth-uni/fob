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
    /// Provider import source for component injection (e.g., "gumbo/mdx", "@mdx-js/react")
    ///
    /// When set, the compiled MDX will include:
    /// ```javascript
    /// import {useMDXComponents as _provideComponents} from '{source}';
    /// ```
    ///
    /// And components will be merged: `_provideComponents()` → `props.components`
    ///
    /// This follows the MDX v3 pattern used by Next.js and @mdx-js/react.
    pub provider_import_source: Option<String>,
}

impl Default for MdxOptions {
    fn default() -> Self {
        Self {
            plugins: Vec::new(),
            jsx_runtime: "react/jsx-runtime".to_string(),
            output_format: OutputFormat::default(),
            frontmatter: None,
            provider_import_source: None,
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

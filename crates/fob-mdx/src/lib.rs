//! # fob-mdx
//!
//! Standalone MDX v3 compiler for Rust.
//!
//! This crate provides a bundler-agnostic MDX compiler that can be integrated
//! with any JavaScript build tool. It parses MDX files, converts them to JSX,
//! and returns all the extracted information (frontmatter, images, exports, etc.)
//! in simple data structures.

pub mod codegen;
pub mod error;
pub mod esm;
pub mod frontmatter;
pub mod nodes;
pub mod options;
pub mod plugins;
pub mod utils;

// Re-export public types
pub use codegen::{mdast_to_jsx, mdast_to_jsx_with_options};
pub use error::MdxError;
pub use frontmatter::{FrontmatterData, FrontmatterFormat, extract_frontmatter};
pub use options::MdxOptions;
pub use plugins::MdxPlugin;

use anyhow::{Result, anyhow};
use bon::Builder;

/// Output format for compiled MDX code
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum OutputFormat {
    /// ES module with import/export (current default)
    #[default]
    Program,
    /// Function body format for runtime eval with new Function()
    FunctionBody,
}

/// Options for MDX compilation
#[derive(Builder)]
pub struct MdxCompileOptions {
    /// Optional filepath for error messages
    #[builder(into)]
    pub filepath: Option<String>,

    /// Enable GitHub Flavored Markdown (tables, strikethrough, task lists).
    /// Enabled by default. Set to `false` to disable.
    #[builder(default = true)]
    pub gfm: bool,

    /// Enable footnote support.
    /// Enabled by default. Set to `false` to disable.
    #[builder(default = true)]
    pub footnotes: bool,

    /// Enable math support (inline `$...$` and block `$$...$$`).
    /// Enabled by default. Set to `false` to disable.
    #[builder(default = true)]
    pub math: bool,

    /// JSX runtime module path
    #[builder(default = "react/jsx-runtime".to_string(), into)]
    pub jsx_runtime: String,

    /// Use default plugins (heading IDs, image optimization).
    /// Enabled by default. Set to `false` to disable.
    #[builder(default = true)]
    pub use_default_plugins: bool,

    /// Additional plugins to apply during compilation.
    /// These are applied AFTER default plugins (if enabled).
    #[builder(default)]
    pub plugins: Vec<Box<dyn MdxPlugin>>,

    /// Output format (Program or FunctionBody)
    #[builder(default)]
    pub output_format: OutputFormat,

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
    #[builder(into)]
    pub provider_import_source: Option<String>,
}

impl std::fmt::Debug for MdxCompileOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MdxCompileOptions")
            .field("filepath", &self.filepath)
            .field("gfm", &self.gfm)
            .field("footnotes", &self.footnotes)
            .field("math", &self.math)
            .field("jsx_runtime", &self.jsx_runtime)
            .field("use_default_plugins", &self.use_default_plugins)
            .field("output_format", &self.output_format)
            .field("provider_import_source", &self.provider_import_source)
            .field("plugins_count", &self.plugins.len())
            .finish()
    }
}

impl Default for MdxCompileOptions {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl MdxCompileOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_jsx_runtime(mut self, jsx_runtime: impl Into<String>) -> Self {
        self.jsx_runtime = jsx_runtime.into();
        self
    }

    pub fn with_plugin(mut self, plugin: Box<dyn MdxPlugin>) -> Self {
        self.plugins.push(plugin);
        self
    }
}

/// Result of MDX compilation
#[derive(Debug, Clone)]
pub struct MdxCompileResult {
    pub code: String,
    pub frontmatter: Option<FrontmatterData>,
    pub images: Vec<String>,
    pub named_exports: Vec<String>,
    pub reexports: Vec<String>,
    pub imports: Vec<String>,
    pub default_export: Option<String>,
}

/// Maximum allowed MDX source size (10MB) to prevent DoS attacks
const MAX_MDX_SIZE: usize = 10 * 1024 * 1024;

/// Compile an MDX string to JSX with optional plugins
pub fn compile(
    source: &str,
    options: MdxCompileOptions,
) -> Result<MdxCompileResult, Box<MdxError>> {
    // Validate input size to prevent DoS
    if source.len() > MAX_MDX_SIZE {
        return Err(Box::new(MdxError::new(format!(
            "MDX source exceeds maximum size of {} bytes ({} MB)",
            MAX_MDX_SIZE,
            MAX_MDX_SIZE / 1024 / 1024
        ))));
    }

    // Set up markdown parser options
    let mut parse_options = markdown::ParseOptions::mdx();

    // Enable ESM parsing with OXC validation
    parse_options.mdx_esm_parse = Some(Box::new(crate::esm::validate_esm_syntax));

    // Enable frontmatter parsing (YAML and TOML)
    parse_options.constructs.frontmatter = true;

    // Enable GFM features if requested
    if options.gfm {
        parse_options.constructs.gfm_strikethrough = true;
        parse_options.constructs.gfm_table = true;
        parse_options.constructs.gfm_task_list_item = true;
        parse_options.constructs.gfm_autolink_literal = true;
    }

    // Enable footnotes if requested
    if options.footnotes {
        parse_options.constructs.gfm_footnote_definition = true;
    }

    // Enable math if requested
    if options.math {
        parse_options.constructs.math_text = true;
        parse_options.constructs.math_flow = true;
    }

    // Parse MDX to markdown AST
    let mdast = markdown::to_mdast(source, &parse_options).map_err(|e| {
        let mut err = MdxError::parse_error(e.to_string());
        if let Some(filepath) = &options.filepath {
            err = err.with_file(filepath.clone());
        }
        Box::new(err)
    })?;

    // Extract frontmatter (removes frontmatter nodes from AST)
    let (cleaned_mdast, frontmatter) =
        extract_frontmatter(&mdast).map_err(|e| Box::new(MdxError::new(format!("{:#}", e))))?;

    // Set up MDX conversion options with plugins and jsx_runtime
    let mut mdx_options = MdxOptions {
        plugins: Vec::new(),
        jsx_runtime: options.jsx_runtime.clone(),
        output_format: options.output_format,
        frontmatter: frontmatter.clone(),
        provider_import_source: options.provider_import_source.clone(),
    };

    // Add default plugins first (if enabled)
    if options.use_default_plugins {
        mdx_options = mdx_options
            .with_plugin(Box::new(plugins::HeadingIdPlugin::default()))
            .with_plugin(Box::new(plugins::ImageOptimizationPlugin::default()));
    }

    // Add user's custom plugins (on top of defaults)
    for plugin in options.plugins {
        mdx_options = mdx_options.with_plugin(plugin);
    }

    // Convert mdast to JSX (applies plugins during conversion)
    let jsx_code = mdast_to_jsx_with_options(&cleaned_mdast, &mdx_options).map_err(|e| {
        let mut err = MdxError::conversion_error(e.to_string());
        if let Some(filepath) = &options.filepath {
            err = err.with_file(filepath.clone());
        }
        Box::new(err)
    })?;

    // Extract collected images from ImageOptimizationPlugin
    let mut images = Vec::new();
    for plugin in &mdx_options.plugins {
        if let Some(image_plugin) = plugin
            .as_any()
            .downcast_ref::<crate::plugins::ImageOptimizationPlugin>()
        {
            images.extend(image_plugin.images());
        }
    }

    // Extract ESM statements from the original AST
    let parsed_exports =
        extract_esm_info(&mdast).map_err(|e| Box::new(MdxError::new(format!("{:#}", e))))?;

    Ok(MdxCompileResult {
        code: jsx_code,
        frontmatter,
        images,
        named_exports: parsed_exports.named_exports,
        reexports: parsed_exports.reexports,
        imports: parsed_exports.imports,
        default_export: parsed_exports.default_export,
    })
}

/// Parsed ES module information from MDX
struct ParsedExports {
    named_exports: Vec<String>,
    reexports: Vec<String>,
    imports: Vec<String>,
    default_export: Option<String>,
}

/// Extract ESM import/export information from the AST
fn extract_esm_info(root: &markdown::mdast::Node) -> Result<ParsedExports> {
    use markdown::mdast::Node;

    let Node::Root(root_node) = root else {
        return Err(anyhow!("Expected Root node"));
    };

    let mut named_exports = Vec::new();
    let mut reexports = Vec::new();
    let mut imports = Vec::new();
    let mut default_export = None;

    for child in &root_node.children {
        if let Node::MdxjsEsm(esm) = child {
            let code = esm.value.trim();

            if crate::esm::is_reexport(code) {
                reexports.push(code.to_string());
            } else if crate::esm::has_named_exports(code) {
                named_exports.push(code.to_string());
            } else if code.starts_with("export default") {
                if let Some(name) = crate::esm::get_default_export_name(code) {
                    default_export = Some(name);
                }
            } else if code.starts_with("import ") {
                imports.push(code.to_string());
            }
        }
    }

    Ok(ParsedExports {
        named_exports,
        reexports,
        imports,
        default_export,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_compilation() {
        let mdx = "# Hello\n\nThis is **bold** text.";
        let result = compile(mdx, MdxCompileOptions::builder().build()).unwrap();

        assert!(result.code.contains("Hello"));
        assert!(result.code.contains("bold"));
    }

    #[test]
    fn test_with_frontmatter() {
        let mdx = "---\ntitle: Test\n---\n\n# Hello";
        let result = compile(mdx, MdxCompileOptions::builder().build()).unwrap();

        assert!(result.frontmatter.is_some());
        let fm = result.frontmatter.unwrap();
        assert_eq!(fm.format, FrontmatterFormat::Yaml);
    }

    #[test]
    fn test_gfm_enabled_by_default() {
        // GFM is now ON by default
        let mdx = "This is ~~strikethrough~~ text.";
        let result = compile(mdx, MdxCompileOptions::builder().build()).unwrap();

        // Should contain del tag for strikethrough
        assert!(result.code.contains("del"));
    }

    #[test]
    fn test_gfm_can_be_disabled() {
        let mdx = "This is ~~strikethrough~~ text.";
        let result = compile(mdx, MdxCompileOptions::builder().gfm(false).build()).unwrap();

        // Should NOT contain del element when GFM is disabled
        // The tildes should be rendered as literal text, not as strikethrough
        assert!(
            !result.code.contains("_components.del"),
            "GFM strikethrough should be disabled"
        );
    }

    #[test]
    fn test_math_enabled_by_default() {
        let mdx = "Inline math: $E = mc^2$";
        let result = compile(mdx, MdxCompileOptions::builder().build()).unwrap();

        // Should contain math spans
        assert!(result.code.contains("math"));
    }

    #[test]
    fn test_math_can_be_disabled() {
        let mdx = "Inline math: $E = mc^2$";
        let result = compile(mdx, MdxCompileOptions::builder().math(false).build()).unwrap();

        // Math delimiters should be rendered as literal text, not math elements
        assert!(!result.code.contains("_components.math"));
    }

    #[test]
    fn test_function_body_output_format() {
        let mdx = "---\ntitle: Test\n---\n\n# Hello";
        let options = MdxCompileOptions::builder()
            .output_format(OutputFormat::FunctionBody)
            .build();
        let result = compile(mdx, options).unwrap();

        // Function-body format should:
        // 1. Start with "use strict"
        assert!(result.code.starts_with("\"use strict\""));
        // 2. Use arguments[0] for jsx runtime
        assert!(result.code.contains("arguments[0]"));
        // 3. Have const frontmatter (not export const)
        assert!(result.code.contains("const frontmatter ="));
        assert!(!result.code.contains("export const frontmatter"));
        // 4. NOT have import statements
        assert!(!result.code.contains("import {"));
        // 5. Have a return statement with exports
        assert!(result.code.contains("return {default: MDXContent"));
        assert!(result.code.contains("frontmatter: frontmatter"));
    }

    #[test]
    fn test_program_output_format_default() {
        let mdx = "# Hello";
        let result = compile(mdx, MdxCompileOptions::builder().build()).unwrap();

        // Program format (default) should have imports and exports
        assert!(result.code.contains("import {"));
        assert!(result.code.contains("export default function MDXContent"));
    }

    #[test]
    fn test_input_size_limit() {
        // Create a source that exceeds the 10MB limit
        let huge = "x".repeat(11 * 1024 * 1024);
        let result = compile(&huge, MdxCompileOptions::builder().build());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("exceeds maximum size"));
    }

    #[test]
    fn test_builder_with_filepath() {
        let options = MdxCompileOptions::builder().filepath("test.mdx").build();

        assert_eq!(options.filepath, Some("test.mdx".to_string()));
    }

    #[test]
    fn test_builder_into_string() {
        // Test that Into<String> works for jsx_runtime
        let options = MdxCompileOptions::builder()
            .jsx_runtime("preact/jsx-runtime")
            .build();

        assert_eq!(options.jsx_runtime, "preact/jsx-runtime");
    }
}

// Bundler integration (requires bundler feature)
#[cfg(feature = "bundler")]
pub mod bundler;
#[cfg(feature = "bundler")]
pub use bundler::FobMdxPlugin;

// Runtime bundling (requires runtime feature)
#[cfg(feature = "runtime")]
pub mod runtime;
#[cfg(feature = "runtime")]
pub use runtime::{BundleMdxOptions, BundleMdxResult, bundle_mdx};

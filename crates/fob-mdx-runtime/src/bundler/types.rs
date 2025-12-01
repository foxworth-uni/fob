//! Types for the bunny runtime bundling API

use bon::Builder;
use fob_mdx::{FrontmatterData, MdxCompileOptions};
use std::collections::HashMap;

/// Options for runtime MDX bundling
///
/// This configures how MDX content should be compiled and bundled at runtime,
/// similar to the mdx-bundler JavaScript library.
///
/// # Example
///
/// ```rust,no_run
/// use fob_mdx_runtime::bundler::{BundleMdxOptions, bundle_mdx};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let options = BundleMdxOptions::builder()
///     .source(r#"
/// # Hello World
///
/// import Button from './Button.tsx'
///
/// <Button>Click me</Button>
///     "#)
///     .build()
///     .with_file("./Button.tsx", r#"
/// export default function Button({children}) {
///     return <button>{children}</button>
/// }
///     "#);
///
/// let result = bundle_mdx(options).await?;
/// println!("Bundled code: {}", result.code);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Default, Builder)]
#[builder(on(String, into))]
pub struct BundleMdxOptions {
    /// The MDX source code to compile and bundle
    pub source: String,

    /// Virtual filesystem: map of file paths to their contents
    ///
    /// When your MDX file imports other files, provide them here.
    /// Paths are relative to the MDX file. Use the `.file()` builder
    /// method to add files one at a time.
    #[builder(default)]
    pub files: HashMap<String, String>,

    /// MDX compilation options (GFM, math, plugins, etc.)
    ///
    /// If `None`, uses default options with all features enabled.
    pub mdx_options: Option<MdxCompileOptions>,
}

impl BundleMdxOptions {
    /// Add a virtual file to the filesystem
    ///
    /// # Example
    ///
    /// ```rust
    /// use fob_mdx_runtime::bundler::BundleMdxOptions;
    ///
    /// let options = BundleMdxOptions::builder()
    ///     .source("import X from './x.js'")
    ///     .build()
    ///     .with_file("./x.js", "export default 'hi'");
    /// ```
    pub fn with_file(mut self, path: impl Into<String>, content: impl Into<String>) -> Self {
        self.files.insert(path.into(), content.into());
        self
    }

    /// Set MDX compilation options
    ///
    /// # Example
    ///
    /// ```rust
    /// use fob_mdx_runtime::bundler::BundleMdxOptions;
    /// use fob_mdx::MdxCompileOptions;
    ///
    /// let options = BundleMdxOptions::builder()
    ///     .source("# Hello")
    ///     .mdx_options(
    ///         MdxCompileOptions::builder()
    ///             .math(false) // Disable math if needed
    ///             .build()
    ///     )
    ///     .build();
    /// ```
    pub fn with_mdx_options(mut self, options: MdxCompileOptions) -> Self {
        self.mdx_options = Some(options);
        self
    }
}

/// Result of runtime MDX bundling
///
/// Contains the executable JavaScript bundle and extracted metadata.
///
/// # Example
///
/// ```rust,no_run
/// use fob_mdx_runtime::bundler::{bundle_mdx, BundleMdxOptions};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let result = bundle_mdx(
///     BundleMdxOptions::builder().source("# Hello").build()
/// ).await?;
///
/// // Send to client
/// println!("Bundle size: {} bytes", result.code.len());
///
/// // Check for frontmatter
/// if let Some(fm) = result.frontmatter {
///     println!("Frontmatter: {:?}", fm.raw);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct BundleMdxResult {
    /// Executable JavaScript bundle
    ///
    /// This is a complete, self-contained bundle that can be executed
    /// in a JavaScript runtime. On the client, use it with `getMDXComponent()`
    /// from mdx-bundler/client or a similar runtime.
    pub code: String,

    /// Parsed frontmatter from the MDX file
    ///
    /// Extracted from YAML or TOML frontmatter blocks at the top of the file.
    pub frontmatter: Option<FrontmatterData>,
}

impl BundleMdxResult {
    /// Get the size of the bundled code in bytes
    pub fn size(&self) -> usize {
        self.code.len()
    }

    /// Check if frontmatter was present
    pub fn has_frontmatter(&self) -> bool {
        self.frontmatter.is_some()
    }
}

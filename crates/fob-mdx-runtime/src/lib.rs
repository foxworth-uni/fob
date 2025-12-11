//! # fob-mdx-runtime
//!
//! Runtime MDX compiler and bundler for Rust.
//!
//! This crate provides a runtime bundling API similar to the JavaScript
//! [mdx-bundler](https://github.com/kentcdodds/mdx-bundler) library. It compiles
//! MDX to JSX and bundles all imports into a single executable JavaScript string
//! that can be sent to clients and executed.
//!
//! ## Features
//!
//! - **Core compilation**: Always available via `compile_mdx()` - compiles MDX to JSX
//! - **Runtime bundling**: Optional `bundler` feature - bundles MDX with dependencies
//!
//! ## Use Cases
//!
//! - **CMS/Database content**: Fetch MDX from a database and bundle at request time
//! - **SSR frameworks**: Compile and bundle MDX on the server for each request
//! - **Preview systems**: Show live previews of MDX content before publishing
//! - **Dynamic content platforms**: Serve personalized MDX content per user
//!
//! ## Comparison with Other Tools
//!
//! | Tool | Purpose | When to Use |
//! |------|---------|-------------|
//! | `fob-mdx-runtime` | Runtime bundling | Server-side, dynamic content from CMS/DB |
//! | `fob-plugin-mdx` | Build-time plugin | Static sites, pre-build all content |
//! | `fob-mdx` | Just MDX compilation | Building custom integrations |
//!
//! ## Example: Compile Only (No Bundling)
//!
//! ```rust,no_run
//! use fob_mdx_runtime::compile_mdx;
//! use fob_mdx::MdxCompileOptions;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mdx = "# Hello World\n\nThis is **bold** text.";
//!     
//!     let result = compile_mdx(mdx, MdxCompileOptions::new())?;
//!     
//!     println!("Compiled JSX: {}", result.code);
//!     Ok(())
//! }
//! ```
//!
//! ## Example: With Bundling (Requires `bundler` feature)
//!
//! ```rust,no_run
//! # #[cfg(feature = "bundler")]
//! use fob_mdx_runtime::{bundle_mdx, BundleMdxOptions};
//! use std::collections::HashMap;
//!
//! # #[cfg(feature = "bundler")]
//! #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // MDX content with imports
//!     let mdx = r#"
//! ---
//! title: My Blog Post
//! ---
//!
//! # Hello World
//!
//! import Button from './Button.tsx'
//!
//! <Button>Click me!</Button>
//!     "#;
//!
//!     // Provide dependencies as virtual files
//!     let options = BundleMdxOptions {
//!         source: mdx.to_string(),
//!         files: HashMap::from([
//!             ("./Button.tsx".into(), r#"
//! export default function Button({children}) {
//!     return <button className="btn">{children}</button>
//! }
//!             "#.into()),
//!         ]),
//!         mdx_options: None, // Uses default (all features enabled)
//!     };
//!
//!     // Bundle at runtime
//!     let result = bundle_mdx(options).await?;
//!
//!     println!("Bundle size: {} bytes", result.size());
//!     println!("Title: {:?}", result.frontmatter);
//!
//!     // Send result.code to client for execution
//!     // Client uses: getMDXComponent(code)
//!
//!     Ok(())
//! # }
//! ```
//!
//! ## Performance Considerations
//!
//! - **Runtime overhead**: Bundling happens at request time, add caching!
//! - **Memory usage**: Bundler runs in-memory
//! - **Scaling**: Use caching layer (Redis, in-memory) for production
//!
//! ## Architecture
//!
//! ```text
//! MDX source
//!    ↓
//! fob-mdx (compile MDX → JSX) [always available]
//!    ↓
//! fob-bundler (bundle JSX + imports → single .js) [optional bundler feature]
//!    ↓
//! Executable JavaScript string
//! ```

// Re-export fob-mdx types and functions (always available)
pub use fob_mdx::{
    FrontmatterData, FrontmatterFormat, MdxCompileOptions, MdxCompileResult, MdxError, compile,
};

// Convenience wrapper for compile function
use anyhow::Result;
use fob_mdx::MdxCompileOptions as Options;

/// Compile MDX to JSX (convenience wrapper)
///
/// This is a simple wrapper around `fob_mdx::compile()` that provides
/// a convenient API for basic MDX compilation without bundling.
///
/// # Example
///
/// ```rust,no_run
/// use fob_mdx_runtime::{compile_mdx, MdxCompileOptions};
///
/// let result = compile_mdx("# Hello", MdxCompileOptions::new())?;
/// println!("JSX: {}", result.code);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn compile_mdx(
    source: &str,
    options: Options,
) -> Result<fob_mdx::MdxCompileResult, Box<fob_mdx::MdxError>> {
    compile(source, options)
}

// Bundling module (only available with "bundler" feature)
#[cfg(feature = "bundler")]
pub mod bundler;

#[cfg(feature = "bundler")]
pub use bundler::{BundleMdxOptions, BundleMdxResult, bundle_mdx};

//! Plugin trait for transforming MDX AST and JSX output
//!
//! The MDX plugin system allows custom transformations of both the markdown AST
//! (before JSX conversion) and the generated JSX string (after conversion).

use anyhow::Result;
use markdown::mdast::Node;
use std::any::Any;

/// Plugin for transforming MDX AST and JSX output
///
/// Implement this trait to create custom MDX transformations. Plugins can
/// modify the markdown AST before JSX generation or transform the JSX string
/// after generation.
///
/// # Thread Safety
///
/// Plugins must be `Send + Sync` because MDX files may be processed in parallel.
/// If you need to accumulate state across transformations, use thread-safe
/// primitives like `Arc<Mutex<T>>` or `Arc<RwLock<T>>`.
///
/// # Performance
///
/// Plugins should be fast (ideally < 1ms per document). For expensive operations,
/// consider collecting metadata during AST transformation and deferring heavy
/// work to a separate build step.
pub trait MdxPlugin: Send + Sync {
    /// Plugin name for debugging and logging
    ///
    /// This name appears in debug logs and error messages. Use a short,
    /// lowercase identifier like "heading-ids" or "image-optimization".
    fn name(&self) -> &'static str;

    /// Transform the markdown AST before JSX conversion
    ///
    /// This method receives a mutable reference to the AST root node, allowing
    /// in-place modifications. The AST follows the mdast (markdown AST) structure
    /// from the `markdown` crate.
    ///
    /// # Default Implementation
    ///
    /// The default implementation does nothing and returns `Ok(())`. Override this
    /// method to perform AST transformations.
    fn transform_ast(&self, ast: &mut Node) -> Result<()> {
        let _ = ast;
        Ok(())
    }

    /// Transform the generated JSX string before bundling
    ///
    /// This method receives the complete JSX output as a string, including imports,
    /// exports, and the MDXContent component. You can modify it in place.
    ///
    /// # Default Implementation
    ///
    /// The default implementation does nothing and returns `Ok(())`. Override this
    /// method to perform JSX transformations.
    ///
    /// # Security Warning
    ///
    /// Be careful when injecting user content into JSX strings. Ensure proper
    /// escaping to prevent XSS vulnerabilities.
    fn transform_jsx(&self, jsx: &mut String) -> Result<()> {
        let _ = jsx;
        Ok(())
    }

    /// Enable downcasting to concrete plugin types
    ///
    /// This method allows the bundler to downcast trait objects to specific
    /// plugin implementations to access plugin-specific data (e.g., collected images).
    ///
    /// # Implementation
    ///
    /// Simply return `self`:
    ///
    /// ```rust,ignore
    /// fn as_any(&self) -> &dyn std::any::Any {
    ///     self
    /// }
    /// ```
    fn as_any(&self) -> &dyn Any;
}

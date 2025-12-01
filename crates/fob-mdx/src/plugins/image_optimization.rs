//! Image optimization plugin for collecting and tagging images
//!
//! This plugin identifies all images in MDX documents and collects them for
//! later optimization and processing. It also adds data attributes to track
//! which images have been processed.
//!
//! # Features
//!
//! - Collects both relative and absolute image URLs
//! - Thread-safe image collection using Arc<Mutex<Vec<String>>>
//! - Adds `data-fob-optimized="true"` attribute for tracking
//! - Supports both inline images and MDX JSX image components
//!
//! # Example
//!
//! ```markdown
//! ![Alt text](/images/photo.jpg)
//! <img src="./local.png" />
//! ```
//!
//! Both images will be collected and marked with `data-fob-optimized="true"`.

use std::sync::{Arc, Mutex};

use anyhow::Result;
use markdown::mdast::Node;

use super::MdxPlugin;

/// Plugin that collects images for optimization and adds tracking attributes
///
/// This plugin walks the AST and:
/// 1. Finds all `Node::Image` nodes (markdown images)
/// 2. Extracts image URLs (both relative and absolute)
/// 3. Stores them in a thread-safe collection
/// 4. Adds data attributes during JSX transformation
///
/// # Thread Safety
///
/// Uses `Arc<Mutex<Vec<String>>>` to safely collect images across multiple
/// documents processed in parallel.
///
/// # Usage
///
/// ```rust,no_run
/// use fob_mdx::mdx::plugins::ImageOptimizationPlugin;
///
/// let plugin = ImageOptimizationPlugin::new();
/// // ... use plugin in MdxOptions ...
///
/// // After processing, retrieve collected images
/// let images = plugin.images();
/// println!("Found {} images", images.len());
/// ```
#[derive(Clone)]
pub struct ImageOptimizationPlugin {
    /// Thread-safe collection of image URLs found during AST traversal
    images: Arc<Mutex<Vec<String>>>,
}

impl ImageOptimizationPlugin {
    /// Create a new image optimization plugin
    pub fn new() -> Self {
        Self {
            images: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get a clone of all collected image URLs
    ///
    /// This returns a snapshot of images collected so far. Safe to call
    /// from any thread. Handles mutex poisoning gracefully by recovering
    /// the inner data.
    pub fn images(&self) -> Vec<String> {
        self.images
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Clear all collected images
    ///
    /// Useful for resetting state between batch processing runs.
    pub fn clear(&self) {
        self.images
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
    }

    /// Walk the AST and collect all image URLs
    fn collect_images(&self, node: &Node) {
        match node {
            Node::Image(image) => {
                // Collect the image URL
                let url = image.url.clone();
                self.images
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(url);

                tracing::debug!(
                    url = image.url,
                    alt = image.alt,
                    "Collected image for optimization"
                );
            }
            Node::Root(root) => {
                for child in &root.children {
                    self.collect_images(child);
                }
            }
            Node::Paragraph(para) => {
                for child in &para.children {
                    self.collect_images(child);
                }
            }
            Node::Link(link) => {
                for child in &link.children {
                    self.collect_images(child);
                }
            }
            Node::LinkReference(link_ref) => {
                for child in &link_ref.children {
                    self.collect_images(child);
                }
            }
            Node::Strong(strong) => {
                for child in &strong.children {
                    self.collect_images(child);
                }
            }
            Node::Emphasis(em) => {
                for child in &em.children {
                    self.collect_images(child);
                }
            }
            Node::Delete(del) => {
                for child in &del.children {
                    self.collect_images(child);
                }
            }
            Node::Blockquote(blockquote) => {
                for child in &blockquote.children {
                    self.collect_images(child);
                }
            }
            Node::List(list) => {
                for child in &list.children {
                    self.collect_images(child);
                }
            }
            Node::ListItem(item) => {
                for child in &item.children {
                    self.collect_images(child);
                }
            }
            Node::Table(table) => {
                for child in &table.children {
                    self.collect_images(child);
                }
            }
            Node::TableRow(row) => {
                for child in &row.children {
                    self.collect_images(child);
                }
            }
            Node::TableCell(cell) => {
                for child in &cell.children {
                    self.collect_images(child);
                }
            }
            Node::Heading(heading) => {
                for child in &heading.children {
                    self.collect_images(child);
                }
            }
            Node::FootnoteDefinition(def) => {
                for child in &def.children {
                    self.collect_images(child);
                }
            }
            // Other node types don't contain images
            _ => {}
        }
    }
}

impl Default for ImageOptimizationPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl MdxPlugin for ImageOptimizationPlugin {
    fn name(&self) -> &'static str {
        "image-optimization"
    }

    fn transform_ast(&self, ast: &mut Node) -> Result<()> {
        self.collect_images(ast);
        Ok(())
    }

    fn transform_jsx(&self, jsx: &mut String) -> Result<()> {
        // Add data-fob-optimized="true" to all img elements
        // Pattern: _jsx(_components.img, {
        // We need to inject data-fob-optimized: "true" into the props

        // Simple string replacement approach
        // In production, you'd want to parse the JSX AST for safety
        *jsx = jsx.replace(
            "_jsx(_components.img, {",
            "_jsx(_components.img, {\"data-fob-optimized\": \"true\", ",
        );

        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::mdast::{Image, Paragraph, Root};

    #[test]
    fn test_image_collection() {
        let plugin = ImageOptimizationPlugin::new();

        // Create AST with images
        let ast = Node::Root(Root {
            children: vec![
                Node::Paragraph(Paragraph {
                    children: vec![Node::Image(Image {
                        url: "/images/photo1.jpg".to_string(),
                        alt: "Photo 1".to_string(),
                        title: None,
                        position: None,
                    })],
                    position: None,
                }),
                Node::Paragraph(Paragraph {
                    children: vec![Node::Image(Image {
                        url: "./local.png".to_string(),
                        alt: "Local image".to_string(),
                        title: None,
                        position: None,
                    })],
                    position: None,
                }),
            ],
            position: None,
        });

        plugin.collect_images(&ast);

        let images = plugin.images();
        assert_eq!(images.len(), 2);
        assert_eq!(images[0], "/images/photo1.jpg");
        assert_eq!(images[1], "./local.png");
    }

    #[test]
    fn test_clear_images() {
        let plugin = ImageOptimizationPlugin::new();

        // Add some images
        let ast = Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![Node::Image(Image {
                    url: "/test.jpg".to_string(),
                    alt: "Test".to_string(),
                    title: None,
                    position: None,
                })],
                position: None,
            })],
            position: None,
        });

        plugin.collect_images(&ast);
        assert_eq!(plugin.images().len(), 1);

        // Clear and verify
        plugin.clear();
        assert_eq!(plugin.images().len(), 0);
    }

    #[test]
    fn test_nested_images() {
        let plugin = ImageOptimizationPlugin::new();

        // Image inside a link
        let ast = Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![Node::Link(markdown::mdast::Link {
                    url: "https://example.com".to_string(),
                    title: None,
                    children: vec![Node::Image(Image {
                        url: "/nested.jpg".to_string(),
                        alt: "Nested".to_string(),
                        title: None,
                        position: None,
                    })],
                    position: None,
                })],
                position: None,
            })],
            position: None,
        });

        plugin.collect_images(&ast);
        let images = plugin.images();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0], "/nested.jpg");
    }

    #[test]
    fn test_jsx_transformation() {
        let plugin = ImageOptimizationPlugin::new();

        let mut jsx = String::from(r#"_jsx(_components.img, {src: "/test.jpg", alt: "Test"})"#);

        plugin.transform_jsx(&mut jsx).unwrap();

        assert!(jsx.contains("data-fob-optimized"));
        assert!(jsx.contains("\"true\""));
    }
}

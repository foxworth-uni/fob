//! Link validation plugin for detecting broken internal links
//!
//! This plugin checks all links in MDX documents and warns about potentially
//! broken internal links. It does not fail compilation, only logs warnings
//! to help developers catch broken links early.
//!
//! # Features
//!
//! - Validates internal links (starting with `/` or `#`)
//! - Warns about empty href attributes
//! - Distinguishes between anchor links and page links
//! - Non-blocking: logs warnings without failing compilation
//!
//! # Example
//!
//! ```markdown
//! [Good external](https://example.com)
//! [Internal page](/docs/intro)
//! [Anchor](#section)
//! [Empty]()
//! ```
//!
//! The plugin will warn about the empty link but allow compilation to proceed.

use anyhow::Result;
use markdown::mdast::Node;

use super::MdxPlugin;

/// Plugin that validates internal links and logs warnings for potential issues
///
/// This plugin performs static analysis of links without accessing the filesystem
/// or network. It warns about:
///
/// - Empty URLs
/// - Suspicious patterns (e.g., malformed anchors)
/// - Links that might be broken (heuristic-based)
///
/// # Non-Blocking Behavior
///
/// This plugin never returns an error. All issues are reported via `tracing::warn!`
/// to avoid breaking the build for link issues that might be false positives.
///
/// # Usage
///
/// ```rust,no_run
/// use fob_mdx::plugins::LinkValidationPlugin;
///
/// let plugin = LinkValidationPlugin::new();
/// ```
#[derive(Default)]
pub struct LinkValidationPlugin {
    // Future enhancement: could collect all links for external validation
}

impl LinkValidationPlugin {
    /// Create a new link validation plugin
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate a single link URL
    ///
    /// Returns true if the link appears valid, false otherwise.
    /// Logs warnings for suspicious patterns.
    pub fn validate_link(&self, url: &str, context: &str) -> bool {
        // Empty URLs
        if url.is_empty() {
            tracing::warn!(
                url = url,
                context = context,
                "Empty link URL detected in MDX"
            );
            return false;
        }

        // Anchor links (#section)
        if url.starts_with('#') {
            if url.len() == 1 {
                tracing::warn!(
                    url = url,
                    context = context,
                    "Anchor link with no target (#) detected"
                );
                return false;
            }
            // Anchor links are generally fine, but we could validate the ID exists
            tracing::debug!(url = url, "Anchor link detected");
            return true;
        }

        // Internal page links (/path/to/page)
        if url.starts_with('/') {
            // Basic validation: check for suspicious patterns
            if url.contains("//") && !url.starts_with("//") {
                tracing::warn!(
                    url = url,
                    context = context,
                    "Internal link contains double slashes (potential typo)"
                );
            }

            if url.ends_with('/') && url.len() > 1 {
                tracing::debug!(
                    url = url,
                    "Internal link ends with slash (might want trailing slash handling)"
                );
            }

            // Check for common file extensions that might indicate broken links
            if url.ends_with(".md") || url.ends_with(".mdx") {
                tracing::warn!(
                    url = url,
                    context = context,
                    "Internal link points to .md/.mdx file (should link to rendered page instead)"
                );
            }

            return true;
        }

        // Relative links (./path or ../path)
        if url.starts_with("./") || url.starts_with("../") {
            tracing::debug!(
                url = url,
                "Relative link detected (ensure relative paths are correct)"
            );
            return true;
        }

        // External links (http://, https://, mailto:, etc.)
        if url.starts_with("http://")
            || url.starts_with("https://")
            || url.starts_with("mailto:")
            || url.starts_with("tel:")
        {
            tracing::debug!(url = url, "External link detected");
            return true;
        }

        // Protocol-relative URLs (//example.com)
        if url.starts_with("//") {
            tracing::debug!(url = url, "Protocol-relative URL detected");
            return true;
        }

        // Other patterns might be valid (e.g., custom protocols, data URIs)
        tracing::debug!(url = url, "Link with non-standard format detected");
        true
    }

    /// Walk the AST and validate all links
    fn validate_links(&self, node: &Node) {
        match node {
            Node::Link(link) => {
                let context = link
                    .children
                    .iter()
                    .filter_map(|child| {
                        if let Node::Text(text) = child {
                            Some(text.value.as_str())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("");

                self.validate_link(&link.url, &context);
            }
            Node::LinkReference(link_ref) => {
                // Link references use a definition elsewhere
                // We'd need to track definitions to validate these properly
                tracing::debug!(
                    identifier = link_ref.identifier,
                    "Link reference detected (validation requires definition tracking)"
                );
            }
            Node::Root(root) => {
                for child in &root.children {
                    self.validate_links(child);
                }
            }
            Node::Paragraph(para) => {
                for child in &para.children {
                    self.validate_links(child);
                }
            }
            Node::Heading(heading) => {
                for child in &heading.children {
                    self.validate_links(child);
                }
            }
            Node::Strong(strong) => {
                for child in &strong.children {
                    self.validate_links(child);
                }
            }
            Node::Emphasis(em) => {
                for child in &em.children {
                    self.validate_links(child);
                }
            }
            Node::Delete(del) => {
                for child in &del.children {
                    self.validate_links(child);
                }
            }
            Node::Blockquote(blockquote) => {
                for child in &blockquote.children {
                    self.validate_links(child);
                }
            }
            Node::List(list) => {
                for child in &list.children {
                    self.validate_links(child);
                }
            }
            Node::ListItem(item) => {
                for child in &item.children {
                    self.validate_links(child);
                }
            }
            Node::Table(table) => {
                for child in &table.children {
                    self.validate_links(child);
                }
            }
            Node::TableRow(row) => {
                for child in &row.children {
                    self.validate_links(child);
                }
            }
            Node::TableCell(cell) => {
                for child in &cell.children {
                    self.validate_links(child);
                }
            }
            Node::FootnoteDefinition(def) => {
                for child in &def.children {
                    self.validate_links(child);
                }
            }
            // Other node types don't contain links
            _ => {}
        }
    }
}

impl MdxPlugin for LinkValidationPlugin {
    fn name(&self) -> &'static str {
        "link-validation"
    }

    fn transform_ast(&self, ast: &mut Node) -> Result<()> {
        self.validate_links(ast);
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    // No JSX transformation needed
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::mdast::{Link, Paragraph, Root, Text};

    #[test]
    fn test_valid_external_link() {
        let plugin = LinkValidationPlugin::new();
        assert!(plugin.validate_link("https://example.com", "Example"));
        assert!(plugin.validate_link("http://example.com", "Example"));
        assert!(plugin.validate_link("mailto:test@example.com", "Email"));
    }

    #[test]
    fn test_valid_internal_link() {
        let plugin = LinkValidationPlugin::new();
        assert!(plugin.validate_link("/docs/intro", "Intro"));
        assert!(plugin.validate_link("/about", "About"));
    }

    #[test]
    fn test_valid_anchor_link() {
        let plugin = LinkValidationPlugin::new();
        assert!(plugin.validate_link("#section", "Section"));
        assert!(plugin.validate_link("#top", "Top"));
    }

    #[test]
    fn test_empty_link() {
        let plugin = LinkValidationPlugin::new();
        assert!(!plugin.validate_link("", "Empty"));
    }

    #[test]
    fn test_empty_anchor() {
        let plugin = LinkValidationPlugin::new();
        assert!(!plugin.validate_link("#", "Empty anchor"));
    }

    #[test]
    fn test_relative_link() {
        let plugin = LinkValidationPlugin::new();
        assert!(plugin.validate_link("./page", "Page"));
        assert!(plugin.validate_link("../other", "Other"));
    }

    #[test]
    fn test_link_traversal() {
        let plugin = LinkValidationPlugin::new();

        let ast = Node::Root(Root {
            children: vec![
                Node::Paragraph(Paragraph {
                    children: vec![Node::Link(Link {
                        url: "https://example.com".to_string(),
                        title: None,
                        children: vec![Node::Text(Text {
                            value: "Example".to_string(),
                            position: None,
                        })],
                        position: None,
                    })],
                    position: None,
                }),
                Node::Paragraph(Paragraph {
                    children: vec![Node::Link(Link {
                        url: "/internal".to_string(),
                        title: None,
                        children: vec![Node::Text(Text {
                            value: "Internal".to_string(),
                            position: None,
                        })],
                        position: None,
                    })],
                    position: None,
                }),
            ],
            position: None,
        });

        // Should not panic or error
        let result = plugin.transform_ast(&mut ast.clone());
        assert!(result.is_ok());
    }

    #[test]
    fn test_mdx_file_warning() {
        let plugin = LinkValidationPlugin::new();
        // This should log a warning but still return true
        assert!(plugin.validate_link("/docs/page.mdx", "MDX Page"));
    }

    #[test]
    fn test_double_slash_warning() {
        let plugin = LinkValidationPlugin::new();
        // This should log a warning but still return true
        assert!(plugin.validate_link("/docs//page", "Double Slash"));
    }

    #[test]
    fn test_protocol_relative_url() {
        let plugin = LinkValidationPlugin::new();
        assert!(plugin.validate_link("//cdn.example.com/script.js", "CDN"));
    }
}

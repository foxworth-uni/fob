//! Frontmatter extraction and parsing

use anyhow::{Context, Result, anyhow};
use markdown::mdast::Node;
use serde_json::Value as JsonValue;

use super::types::{FrontmatterData, FrontmatterFormat};

/// Extract and parse frontmatter from an MDX AST
///
/// Searches for YAML or TOML frontmatter nodes in the AST root and parses them
/// into JSON for build-time access. This allows frontmatter to be processed during
/// compilation rather than at runtime.
///
/// # Security
///
/// - YAML parsing uses serde-saphyr which is panic-free for untrusted input
/// - TOML parsing uses the toml crate which is memory-safe
/// - Frontmatter is parsed and validated before being embedded in output
///
/// # Arguments
///
/// * `root` - The root node of the markdown AST
///
/// # Returns
///
/// A tuple of `(cleaned_ast, Option<FrontmatterData>)` where:
/// - `cleaned_ast` is the AST with frontmatter nodes removed
/// - `FrontmatterData` contains the parsed frontmatter if present
///
/// # Errors
///
/// Returns an error if:
/// - The input is not a Root node
/// - Frontmatter parsing fails (invalid YAML/TOML syntax)
/// - Multiple frontmatter blocks are found (only one is allowed)
pub fn extract_frontmatter(root: &Node) -> Result<(Node, Option<FrontmatterData>)> {
    let Node::Root(root_node) = root else {
        return Err(anyhow!("Expected Root node, got {:?}", root));
    };

    let mut frontmatter: Option<FrontmatterData> = None;
    let mut cleaned_children = Vec::new();

    for child in &root_node.children {
        match child {
            Node::Yaml(yaml_node) => {
                if frontmatter.is_some() {
                    return Err(anyhow!(
                        "Multiple frontmatter blocks found. Only one frontmatter block is allowed per MDX file."
                    ));
                }

                // Parse YAML to JSON
                let data: JsonValue = serde_saphyr::from_str(&yaml_node.value)
                    .context("Failed to parse YAML frontmatter")?;

                frontmatter = Some(FrontmatterData::new(
                    FrontmatterFormat::Yaml,
                    data,
                    yaml_node.value.clone(),
                ));
            }
            Node::Toml(toml_node) => {
                if frontmatter.is_some() {
                    return Err(anyhow!(
                        "Multiple frontmatter blocks found. Only one frontmatter block is allowed per MDX file."
                    ));
                }

                // Parse TOML to JSON via serde
                let data: toml::Value =
                    toml::from_str(&toml_node.value).context("Failed to parse TOML frontmatter")?;

                // Convert TOML value to JSON value
                let json_data =
                    serde_json::to_value(&data).context("Failed to convert TOML to JSON")?;

                frontmatter = Some(FrontmatterData::new(
                    FrontmatterFormat::Toml,
                    json_data,
                    toml_node.value.clone(),
                ));
            }
            other => {
                // Keep all non-frontmatter nodes
                cleaned_children.push(other.clone());
            }
        }
    }

    // Create new root node without frontmatter
    let cleaned_root = Node::Root(markdown::mdast::Root {
        children: cleaned_children,
        position: root_node.position.clone(),
    });

    Ok((cleaned_root, frontmatter))
}

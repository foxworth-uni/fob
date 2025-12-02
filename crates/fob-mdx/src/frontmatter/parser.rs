//! Frontmatter extraction and parsing

use anyhow::{Context, Result, anyhow};
use markdown::mdast::Node;
use serde_json::Value as JsonValue;

use super::props::PropDefinition;
use super::props_parser::parse_prop_expression;
use super::types::{FrontmatterData, FrontmatterFormat};

/// Parse props from frontmatter `props:` section
fn parse_props_from_data(data: &JsonValue) -> Result<Vec<PropDefinition>> {
    let Some(props_value) = data.get("props") else {
        return Ok(Vec::new());
    };

    let Some(props_obj) = props_value.as_object() else {
        return Ok(Vec::new());
    };

    let mut props = Vec::new();
    for (name, expr) in props_obj {
        if let Some(expr_str) = expr.as_str() {
            let prop = parse_prop_expression(name, expr_str)
                .map_err(|e| anyhow!("Failed to parse prop '{}': {}", name, e))?;
            props.push(prop);
        }
    }
    Ok(props)
}

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

                // Parse props from frontmatter
                let props = parse_props_from_data(&data)?;

                frontmatter = Some(
                    FrontmatterData::new(FrontmatterFormat::Yaml, data, yaml_node.value.clone())
                        .with_props(props),
                );
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

                // Parse props from frontmatter
                let props = parse_props_from_data(&json_data)?;

                frontmatter = Some(
                    FrontmatterData::new(
                        FrontmatterFormat::Toml,
                        json_data,
                        toml_node.value.clone(),
                    )
                    .with_props(props),
                );
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

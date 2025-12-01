//! MDX node conversion to JSX
//!
//! Handles conversion of markdown AST nodes to JSX code strings.

mod block;
mod code;
mod inline;
mod list;
mod mdx;
mod table;

use anyhow::Result;
use markdown::mdast::Node;

use crate::codegen::{CodegenContext, JsValue};

pub use block::*;
pub use code::*;
pub use inline::*;
pub use list::*;
pub use mdx::*;
pub use table::*;

/// Convert a single mdast node to JSX
///
/// Central dispatcher that routes nodes to specialized handlers
pub fn node_to_jsx(
    node: &Node,
    ctx: &mut CodegenContext,
    in_list: bool,
) -> Result<Option<JsValue>> {
    match node {
        // MDX JSX elements
        Node::MdxJsxFlowElement(element) => Ok(Some(jsx_flow_element_to_string(element, ctx)?)),
        Node::MdxJsxTextElement(element) => Ok(Some(jsx_text_element_to_string(element, ctx)?)),

        // MDX expressions
        Node::MdxFlowExpression(expr) => Ok(Some(JsValue::raw(expr.value.clone()))),
        Node::MdxTextExpression(expr) => Ok(Some(JsValue::raw(expr.value.clone()))),

        // Block elements
        Node::Heading(heading) => block::heading_to_jsx(heading, ctx),
        Node::Paragraph(para) => block::paragraph_to_jsx(para, ctx),
        Node::Blockquote(quote) => block::blockquote_to_jsx(quote, ctx),
        Node::ThematicBreak(_) => Ok(Some(JsValue::raw(
            "_jsx(_components.hr, {...props})".to_string(),
        ))),

        // Code blocks
        Node::Code(code) => code::code_block_to_jsx(code, ctx),

        // Lists
        Node::List(list) => list::list_to_jsx(list, ctx),
        Node::ListItem(item) => list::list_item_to_jsx(item, ctx, in_list),

        // Inline elements
        Node::Text(text) => Ok(Some(JsValue::text(text.value.clone()))),
        Node::InlineCode(code) => inline::inline_code_to_jsx(code, ctx),
        Node::Emphasis(emph) => inline::emphasis_to_jsx(emph, ctx),
        Node::Strong(strong) => inline::strong_to_jsx(strong, ctx),
        Node::Link(link) => inline::link_to_jsx(link, ctx),
        Node::Image(image) => inline::image_to_jsx(image, ctx),
        Node::Break(_) => Ok(Some(JsValue::raw(
            "_jsx(_components.br, {...props})".to_string(),
        ))),
        Node::Delete(del) => inline::delete_to_jsx(del, ctx),

        // Tables
        Node::Table(table) => table::table_to_jsx(table, ctx),
        Node::TableRow(row) => table::table_row_to_jsx(row, ctx),
        Node::TableCell(cell) => table::table_cell_to_jsx(cell, ctx),

        // Math
        Node::Math(math) => inline::math_to_jsx(math, ctx),
        Node::InlineMath(inline_math) => inline::inline_math_to_jsx(inline_math, ctx),

        // Footnotes
        Node::FootnoteReference(footnote_ref) => {
            inline::footnote_reference_to_jsx(footnote_ref, ctx)
        }
        Node::FootnoteDefinition(footnote_def) => {
            inline::footnote_definition_to_jsx(footnote_def, ctx)
        }

        // HTML (pass through if safe)
        Node::Html(html) => Ok(Some(JsValue::raw(html.value.clone()))),

        // Skip these nodes
        Node::Definition(_) | Node::Yaml(_) | Node::Toml(_) => Ok(None),

        // Fallback for unhandled nodes
        _ => Ok(None),
    }
}

/// Convert list of child nodes to JSX
///
/// Returns a JsValue that correctly handles text-only, mixed content, and complex scenarios.
pub fn children_to_jsx(children: &[Node], ctx: &mut CodegenContext) -> Result<JsValue> {
    if children.is_empty() {
        return Ok(JsValue::text(""));
    }

    // Collect all child values
    let values: Result<Vec<JsValue>> = children
        .iter()
        .filter_map(|child| node_to_jsx(child, ctx, false).transpose())
        .collect();
    let values = values?;

    if values.is_empty() {
        return Ok(JsValue::text(""));
    }

    // If single value, return it directly
    if values.len() == 1 {
        return Ok(values.into_iter().next().unwrap());
    }

    // Multiple values: check if we need an array
    let has_non_text = values
        .iter()
        .any(|v| matches!(v, JsValue::Raw(_) | JsValue::Array(_)));

    if has_non_text {
        // Mixed content: return as array
        Ok(JsValue::array(values))
    } else {
        // All text: concatenate
        let text: String = values
            .iter()
            .filter_map(|v| match v {
                JsValue::Text(s) => Some(s.as_str()),
                _ => None,
            })
            .collect();
        Ok(JsValue::text(text))
    }
}

/// Convert list of child nodes to JSX array (comma-separated for _jsxs)
pub fn children_to_jsx_array(
    children: &[Node],
    ctx: &mut CodegenContext,
    in_list: bool,
) -> Result<String> {
    let items: Result<Vec<String>> = children
        .iter()
        .filter_map(|child| {
            node_to_jsx(child, ctx, in_list)
                .transpose()
                .map(|result| result.map(|v| v.to_js()))
        })
        .collect();

    Ok(items?.join(", "))
}

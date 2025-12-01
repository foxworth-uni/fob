//! Block-level node conversions (headings, paragraphs, blockquotes)

use anyhow::Result;
use markdown::mdast::{Blockquote, Heading, Paragraph};

use super::children_to_jsx;
use crate::codegen::{CodegenContext, JsValue};

/// Convert heading node to JSX
pub fn heading_to_jsx(heading: &Heading, ctx: &mut CodegenContext) -> Result<Option<JsValue>> {
    let level = heading.depth;
    let children = children_to_jsx(&heading.children, ctx)?;
    let jsx = format!(
        "_jsx(_components.h{}, {{...props, children: {}}})",
        level,
        children.to_js()
    );
    Ok(Some(JsValue::raw(jsx)))
}

/// Convert paragraph node to JSX
pub fn paragraph_to_jsx(para: &Paragraph, ctx: &mut CodegenContext) -> Result<Option<JsValue>> {
    let children = children_to_jsx(&para.children, ctx)?;
    let jsx = format!(
        "_jsx(_components.p, {{...props, children: {}}})",
        children.to_js()
    );
    Ok(Some(JsValue::raw(jsx)))
}

/// Convert blockquote node to JSX
pub fn blockquote_to_jsx(quote: &Blockquote, ctx: &mut CodegenContext) -> Result<Option<JsValue>> {
    let children = children_to_jsx(&quote.children, ctx)?;
    let jsx = format!(
        "_jsx(_components.blockquote, {{...props, children: {}}})",
        children.to_js()
    );
    Ok(Some(JsValue::raw(jsx)))
}

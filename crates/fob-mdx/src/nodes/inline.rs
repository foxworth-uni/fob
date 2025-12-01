//! Inline node conversions (emphasis, strong, links, images, etc.)

use anyhow::Result;
use markdown::mdast::{
    Delete, Emphasis, FootnoteDefinition, FootnoteReference, Image, InlineCode, InlineMath, Link,
    Math, Strong,
};

use super::children_to_jsx;
use crate::codegen::{CodegenContext, JsValue, escape_js_string};

/// Convert inline code node to JSX
pub fn inline_code_to_jsx(code: &InlineCode, _ctx: &mut CodegenContext) -> Result<Option<JsValue>> {
    let escaped = escape_js_string(&code.value);
    let jsx = format!(
        "_jsx(_components.code, {{...props, children: \"{}\"}})",
        escaped
    );
    Ok(Some(JsValue::raw(jsx)))
}

/// Convert emphasis node to JSX
pub fn emphasis_to_jsx(emph: &Emphasis, ctx: &mut CodegenContext) -> Result<Option<JsValue>> {
    let children = children_to_jsx(&emph.children, ctx)?;
    let jsx = format!(
        "_jsx(_components.em, {{...props, children: {}}})",
        children.to_js()
    );
    Ok(Some(JsValue::raw(jsx)))
}

/// Convert strong node to JSX
pub fn strong_to_jsx(strong: &Strong, ctx: &mut CodegenContext) -> Result<Option<JsValue>> {
    let children = children_to_jsx(&strong.children, ctx)?;
    let jsx = format!(
        "_jsx(_components.strong, {{...props, children: {}}})",
        children.to_js()
    );
    Ok(Some(JsValue::raw(jsx)))
}

/// Convert link node to JSX
pub fn link_to_jsx(link: &Link, ctx: &mut CodegenContext) -> Result<Option<JsValue>> {
    let href = escape_js_string(&link.url);
    let children = children_to_jsx(&link.children, ctx)?;
    let jsx = format!(
        "_jsx(_components.a, {{...props, href: \"{}\", children: {}}})",
        href,
        children.to_js()
    );
    Ok(Some(JsValue::raw(jsx)))
}

/// Convert image node to JSX
pub fn image_to_jsx(image: &Image, _ctx: &mut CodegenContext) -> Result<Option<JsValue>> {
    let src = escape_js_string(&image.url);
    let alt = escape_js_string(&image.alt);

    // Include title attribute if present (shows as tooltip on hover)
    let jsx = if let Some(title) = &image.title {
        let title = escape_js_string(title);
        format!(
            "_jsx(_components.img, {{...props, src: \"{}\", alt: \"{}\", title: \"{}\"}})",
            src, alt, title
        )
    } else {
        format!(
            "_jsx(_components.img, {{...props, src: \"{}\", alt: \"{}\"}})",
            src, alt
        )
    };

    Ok(Some(JsValue::raw(jsx)))
}

/// Convert delete (strikethrough) node to JSX
pub fn delete_to_jsx(del: &Delete, ctx: &mut CodegenContext) -> Result<Option<JsValue>> {
    let children = children_to_jsx(&del.children, ctx)?;
    let jsx = format!(
        "_jsx(_components.del, {{...props, children: {}}})",
        children.to_js()
    );
    Ok(Some(JsValue::raw(jsx)))
}

/// Convert footnote reference to JSX
pub fn footnote_reference_to_jsx(
    footnote_ref: &FootnoteReference,
    _ctx: &mut CodegenContext,
) -> Result<Option<JsValue>> {
    let id = &footnote_ref.identifier;
    let label = footnote_ref.label.as_deref().unwrap_or(id);
    let jsx = format!(
        "_jsx(_components.sup, {{...props, children: _jsx(_components.a, {{href: \"#fn-{}\", id: \"fnref-{}\", children: \"{}\"}})}})",
        id,
        id,
        escape_js_string(label)
    );
    Ok(Some(JsValue::raw(jsx)))
}

/// Convert footnote definition to JSX
pub fn footnote_definition_to_jsx(
    footnote_def: &FootnoteDefinition,
    ctx: &mut CodegenContext,
) -> Result<Option<JsValue>> {
    let id = &footnote_def.identifier;
    let label = footnote_def.label.as_deref().unwrap_or(id);
    let children = children_to_jsx(&footnote_def.children, ctx)?;
    let jsx = format!(
        "_jsx(_components.div, {{...props, id: \"fn-{}\", children: _jsx(_components.p, {{children: [\"{}\", \". \", {}, \" \", _jsx(_components.a, {{href: \"#fnref-{}\", children: \"\u{21a9}\"}})]}})}}))",
        id,
        escape_js_string(label),
        children.to_js(),
        id
    );
    Ok(Some(JsValue::raw(jsx)))
}

/// Convert block math node to JSX
pub fn math_to_jsx(math: &Math, _ctx: &mut CodegenContext) -> Result<Option<JsValue>> {
    let value = escape_js_string(&math.value);
    let jsx = format!(
        "_jsx(_components.span, {{...props, className: \"math math-display\", children: \"{}\"}})",
        value
    );
    Ok(Some(JsValue::raw(jsx)))
}

/// Convert inline math node to JSX
pub fn inline_math_to_jsx(
    inline_math: &InlineMath,
    _ctx: &mut CodegenContext,
) -> Result<Option<JsValue>> {
    let value = escape_js_string(&inline_math.value);
    let jsx = format!(
        "_jsx(_components.span, {{...props, className: \"math math-inline\", children: \"{}\"}})",
        value
    );
    Ok(Some(JsValue::raw(jsx)))
}

//! MDX-specific node conversions (JSX elements, expressions)

use anyhow::Result;
use markdown::mdast::{MdxJsxFlowElement, MdxJsxTextElement, Node};

use super::children_to_jsx;
use crate::codegen::{CodegenContext, JsValue, escape_js_string};

/// Check if component name should use _components map
fn should_use_components_map(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut chars = name.chars();
    if let Some(first) = chars.next() {
        if first.is_ascii_uppercase() || first == '_' || first == '$' {
            return true;
        }
    }
    name.contains('.')
}

/// Check if a string is a valid JavaScript identifier
fn is_valid_js_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// Format component access expression (_components.Foo or _components["foo-bar"])
fn format_component_access(name: &str) -> String {
    let parts: Vec<&str> = name.split('.').collect();
    if parts.is_empty() {
        return "_components".to_string();
    }

    if parts.iter().any(|part| !is_valid_js_identifier(part)) {
        return format!("_components[\"{}\"]", escape_js_string(name));
    }

    let mut expr = String::from("_components");
    for (index, part) in parts.iter().enumerate() {
        if index == 0 {
            expr.push('.');
            expr.push_str(part);
        } else {
            expr = format!("(({}) || {{}}).{}", expr, part);
        }
    }
    expr
}

/// Convert MDX JSX flow element to JsValue
pub fn jsx_flow_element_to_string(
    element: &MdxJsxFlowElement,
    ctx: &mut CodegenContext,
) -> Result<JsValue> {
    jsx_element_to_string(&element.name, &element.attributes, &element.children, ctx)
}

/// Convert MDX JSX text element to JsValue
pub fn jsx_text_element_to_string(
    element: &MdxJsxTextElement,
    ctx: &mut CodegenContext,
) -> Result<JsValue> {
    jsx_element_to_string(&element.name, &element.attributes, &element.children, ctx)
}

/// Generic JSX element converter (generates _jsx calls)
fn jsx_element_to_string(
    name: &Option<String>,
    attributes: &[markdown::mdast::AttributeContent],
    children: &[Node],
    ctx: &mut CodegenContext,
) -> Result<JsValue> {
    let tag_expr = name
        .as_ref()
        .map(|n| {
            // Check if this component was imported directly
            let is_imported = ctx.imported_components.contains(n);
            let uses_component_map = should_use_components_map(n);

            // Debug logging to diagnose MDX component resolution
            tracing::debug!(
                component_name = n,
                is_imported = is_imported,
                uses_component_map = uses_component_map,
                all_imported = ?ctx.imported_components,
                "Resolving JSX component reference"
            );

            if is_imported {
                // Use the imported component directly (e.g., Button)
                n.clone()
            } else if uses_component_map {
                // Use _components map for provider-injected components
                // No fallback - if component not provided, let React handle undefined
                // This avoids creating invalid HTML tags like <CustomAlert>
                format_component_access(n)
            } else {
                // HTML element
                format!("\"{}\"", escape_js_string(n))
            }
        })
        .unwrap_or_else(|| "\"div\"".to_string());

    // Convert attributes to props object
    let mut props = Vec::new();

    for attr in attributes {
        match attr {
            markdown::mdast::AttributeContent::Property(prop) => {
                let prop_name = &prop.name;
                let prop_value = match &prop.value {
                    Some(markdown::mdast::AttributeValue::Literal(lit)) => {
                        format!("\"{}\"", escape_js_string(lit))
                    }
                    Some(markdown::mdast::AttributeValue::Expression(expr)) => expr.value.clone(),
                    None => "true".to_string(),
                };
                // Quote prop names that aren't valid JS identifiers (kebab-case, etc.)
                let key = if is_valid_js_identifier(prop_name) {
                    prop_name.clone()
                } else {
                    format!("\"{}\"", escape_js_string(prop_name))
                };
                props.push(format!("{}: {}", key, prop_value));
            }
            markdown::mdast::AttributeContent::Expression(expr) => {
                // Spread expression
                props.push(format!("...{}", expr.value));
            }
        }
    }

    // Handle children
    let jsx = if children.is_empty() {
        // Self-closing: _jsx(Tag, {props})
        if props.is_empty() {
            format!("_jsx({}, {{}})", tag_expr)
        } else {
            format!("_jsx({}, {{{}}})", tag_expr, props.join(", "))
        }
    } else {
        // With children: _jsx(Tag, {props, children: ...})
        let children_value = children_to_jsx(children, ctx)?;
        props.push(format!("children: {}", children_value.to_js()));

        if children.len() == 1 {
            format!("_jsx({}, {{{}}})", tag_expr, props.join(", "))
        } else {
            format!("_jsxs({}, {{{}}})", tag_expr, props.join(", "))
        }
    };

    Ok(JsValue::raw(jsx))
}

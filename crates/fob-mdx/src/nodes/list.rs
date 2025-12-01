//! List node conversions (ordered, unordered, task lists)

use anyhow::Result;
use markdown::mdast::{List, ListItem};

use super::{children_to_jsx, children_to_jsx_array};
use crate::codegen::{CodegenContext, JsValue};

/// Convert list node to JSX
pub fn list_to_jsx(list: &List, ctx: &mut CodegenContext) -> Result<Option<JsValue>> {
    let tag = if list.ordered { "ol" } else { "ul" };
    let children = children_to_jsx_array(&list.children, ctx, true)?;
    let jsx = format!(
        "_jsxs(_components.{}, {{...props, children: [{}]}})",
        tag, children
    );
    Ok(Some(JsValue::raw(jsx)))
}

/// Convert list item node to JSX
pub fn list_item_to_jsx(
    item: &ListItem,
    ctx: &mut CodegenContext,
    in_list: bool,
) -> Result<Option<JsValue>> {
    let children_value = children_to_jsx(&item.children, ctx)?;

    // Task list item support: render a disabled checkbox when `checked` is present
    let jsx;
    if let Some(checked) = item.checked {
        // Build children array: [<input .../>, " ", ...original children]
        let task_id = ctx.next_key();
        let checkbox = format!(
            "_jsx(_components.input, {{type: \"checkbox\", checked: {}, \"data-task-id\": \"{}\", onChange: _handleTaskToggle}})",
            if checked { "true" } else { "false" },
            task_id
        );
        let mut parts: Vec<String> = Vec::new();
        parts.push(checkbox);
        parts.push("\" \"".to_string());
        match children_value {
            JsValue::Array(items) => {
                parts.extend(items.into_iter().map(|v| v.to_js()));
            }
            other => parts.push(other.to_js()),
        }
        let children = parts.join(", ");
        if in_list {
            let key = ctx.next_key();
            jsx = format!(
                "_jsxs(_components.li, {{...props, children: [{}]}}, \"{}\")",
                children, key
            );
        } else {
            jsx = format!(
                "_jsxs(_components.li, {{...props, children: [{}]}})",
                children
            );
        }
    } else {
        // Regular list item
        if in_list {
            let key = ctx.next_key();
            jsx = format!(
                "_jsx(_components.li, {{...props, children: {}}}, \"{}\")",
                children_value.to_js(),
                key
            );
        } else {
            jsx = format!(
                "_jsx(_components.li, {{...props, children: {}}})",
                children_value.to_js()
            );
        }
    }
    Ok(Some(JsValue::raw(jsx)))
}

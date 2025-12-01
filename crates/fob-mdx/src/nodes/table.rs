//! Table node conversions

use anyhow::Result;
use markdown::mdast::{AlignKind, Table, TableCell, TableRow};

use super::{children_to_jsx, children_to_jsx_array};
use crate::codegen::{CodegenContext, JsValue};

/// Convert table node to JSX
pub fn table_to_jsx(table: &Table, ctx: &mut CodegenContext) -> Result<Option<JsValue>> {
    ctx.enter_table(Some(table.align.clone()));

    if table.children.is_empty() {
        ctx.exit_table();
        let jsx = "_jsx(_components.table, {...props, children: []})";
        return Ok(Some(JsValue::raw(jsx.to_string())));
    }

    let mut sections = Vec::new();

    if let Some(header_row) = table.children.first() {
        if let Some(jsx) = super::node_to_jsx(header_row, ctx, false)? {
            sections.push(format!(
                "_jsx(_components.thead, {{children: {}}})",
                jsx.to_js()
            ));
        }
    }

    if table.children.len() > 1 {
        let body_rows = &table.children[1..];
        let body_content = children_to_jsx_array(body_rows, ctx, false)?;
        sections.push(format!(
            "_jsx(_components.tbody, {{children: [{}]}})",
            body_content
        ));
    }

    ctx.exit_table();

    let children = if sections.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", sections.join(", "))
    };

    let jsx = format!(
        "_jsxs(_components.table, {{...props, children: {}}})",
        children
    );
    Ok(Some(JsValue::raw(jsx)))
}

/// Convert table row node to JSX
pub fn table_row_to_jsx(row: &TableRow, ctx: &mut CodegenContext) -> Result<Option<JsValue>> {
    ctx.start_table_row();
    let children = children_to_jsx_array(&row.children, ctx, false)?;
    ctx.end_table_row();
    let key = ctx.next_key();
    let jsx = format!(
        "_jsxs(_components.tr, {{...props, children: [{}]}}, \"{}\")",
        children, key
    );
    Ok(Some(JsValue::raw(jsx)))
}

/// Convert table cell node to JSX
pub fn table_cell_to_jsx(cell: &TableCell, ctx: &mut CodegenContext) -> Result<Option<JsValue>> {
    let children = children_to_jsx(&cell.children, ctx)?;
    let key = ctx.next_key();
    let tag = if ctx.is_header_row() { "th" } else { "td" };

    let mut props_parts = Vec::new();

    if let Some(align) = ctx.current_cell_alignment() {
        let text_align = match align {
            AlignKind::Left => Some("left"),
            AlignKind::Right => Some("right"),
            AlignKind::Center => Some("center"),
            AlignKind::None => None,
        };

        if let Some(value) = text_align {
            props_parts.push(format!("style: {{textAlign: \"{}\"}}", value));
        }
    }

    props_parts.push("...props".to_string());
    props_parts.push(format!("children: {}", children.to_js()));

    ctx.next_table_cell();

    let jsx = format!(
        "_jsx(_components.{}, {{{}}}, \"{}\")",
        tag,
        props_parts.join(", "),
        key
    );
    Ok(Some(JsValue::raw(jsx)))
}

//! Code generation context

use markdown::mdast::AlignKind;
use std::collections::HashSet;

/// Table-specific state used while generating table markup
#[derive(Clone, Debug, Default)]
pub struct TableContext {
    pub alignments: Vec<AlignKind>,
    pub row_index: usize,
    pub col_index: usize,
}

/// Context for tracking position-based keys and table state
#[derive(Default)]
pub struct CodegenContext {
    pub key_counter: usize,
    pub table_stack: Vec<TableContext>,

    /// Components imported directly in the MDX file.
    ///
    /// These components should NOT go through the _components map,
    /// but instead be referenced directly by their imported name.
    ///
    /// Example: If the MDX file has `import { Button } from './ui'`,
    /// then "Button" will be in this set, and `<Button>` will compile
    /// to `_jsx(Button, ...)` instead of `_jsx(_components.Button, ...)`.
    pub imported_components: HashSet<String>,
}

impl CodegenContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enter_table(&mut self, align: Option<Vec<AlignKind>>) {
        self.table_stack.push(TableContext {
            alignments: align.unwrap_or_default(),
            row_index: 0,
            col_index: 0,
        });
    }

    pub fn exit_table(&mut self) {
        self.table_stack.pop();
    }

    pub fn start_table_row(&mut self) {
        if let Some(ctx) = self.table_stack.last_mut() {
            ctx.col_index = 0;
        }
    }

    pub fn end_table_row(&mut self) {
        if let Some(ctx) = self.table_stack.last_mut() {
            ctx.row_index += 1;
        }
    }

    pub fn next_table_cell(&mut self) {
        if let Some(ctx) = self.table_stack.last_mut() {
            ctx.col_index += 1;
        }
    }

    pub fn is_header_row(&self) -> bool {
        self.table_stack
            .last()
            .map(|ctx| ctx.row_index == 0)
            .unwrap_or(false)
    }

    pub fn current_cell_alignment(&self) -> Option<&AlignKind> {
        self.table_stack
            .last()
            .and_then(|ctx| ctx.alignments.get(ctx.col_index))
    }

    pub fn next_key(&mut self) -> String {
        let key = format!("mdx-{}", self.key_counter);
        self.key_counter += 1;
        key
    }
}

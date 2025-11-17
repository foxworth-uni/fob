use std::fmt::Write;

use crate::model::{Documentation, ExportedSymbol, SymbolKind};

/// Render documentation as GitHub-flavoured Markdown.
pub fn render_markdown(doc: &Documentation) -> String {
    let mut output = String::new();

    if doc.modules.is_empty() {
        writeln!(&mut output, "# Documentation").ok();
        writeln!(&mut output, "\n_No documented modules found._").ok();
        return output;
    }

    for (module_index, module) in doc.modules.iter().enumerate() {
        if module_index > 0 {
            output.push('\n');
        }
        let _ = writeln!(&mut output, "# {}", module.path);

        if let Some(description) = module.description.as_ref().and_then(non_empty) {
            let _ = writeln!(&mut output, "\n{}\n", description);
        } else {
            output.push('\n');
        }

        if module.symbols.is_empty() {
            let _ = writeln!(&mut output, "_No documented exports._");
            continue;
        }

        for symbol in &module.symbols {
            render_symbol(&mut output, symbol);
        }
    }

    output
}

fn render_symbol(buffer: &mut String, symbol: &ExportedSymbol) {
    let _ = writeln!(
        buffer,
        "## `{}` ({})",
        symbol.name,
        display_symbol_kind(symbol.kind)
    );
    let _ = writeln!(
        buffer,
        "> Location: line {}, column {}",
        symbol.location.line, symbol.location.column
    );

    if let Some(summary) = symbol.summary.as_ref().and_then(non_empty) {
        let _ = writeln!(buffer, "\n{}\n", summary);
    } else {
        buffer.push('\n');
    }

    if !symbol.parameters.is_empty() {
        let _ = writeln!(buffer, "**Parameters**");
        for parameter in &symbol.parameters {
            let mut line = format!("- `{}`", parameter.name);
            if let Some(ty) = parameter.type_hint.as_ref().and_then(non_empty) {
                line.push_str(": ");
                line.push_str(ty);
            }
            if let Some(description) = parameter.description.as_ref().and_then(non_empty) {
                if !line.ends_with(' ') {
                    line.push(' ');
                }
                line.push_str(description);
            }
            let _ = writeln!(buffer, "{line}");
        }
        buffer.push('\n');
    }

    if let Some(returns) = symbol.returns.as_ref().and_then(non_empty) {
        let _ = writeln!(buffer, "**Returns**");
        let _ = writeln!(buffer, "{}\n", returns);
    }

    if let Some(deprecated) = symbol.deprecated.as_ref().and_then(non_empty) {
        let _ = writeln!(buffer, "> Deprecated: {}", deprecated);
        buffer.push('\n');
    }

    if !symbol.examples.is_empty() {
        let _ = writeln!(buffer, "**Examples**");
        for example in &symbol.examples {
            let _ = writeln!(buffer, "```ts");
            let _ = writeln!(buffer, "{}", example.trim_end());
            let _ = writeln!(buffer, "```");
        }
        buffer.push('\n');
    }

    if !symbol.tags.is_empty() {
        let _ = writeln!(buffer, "**Tags**");
        for tag in &symbol.tags {
            let mut line = format!("- @{}", tag.tag);
            if let Some(name) = tag.name.as_ref().and_then(non_empty) {
                line.push(' ');
                line.push_str(name);
            }
            if let Some(description) = tag.description.as_ref().and_then(non_empty) {
                if !line.ends_with(' ') {
                    line.push(' ');
                }
                line.push_str(description);
            }
            let _ = writeln!(buffer, "{line}");
        }
        buffer.push('\n');
    }
}

fn display_symbol_kind(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Function => "function",
        SymbolKind::Class => "class",
        SymbolKind::Interface => "interface",
        SymbolKind::TypeAlias => "type",
        SymbolKind::Enum => "enum",
        SymbolKind::Variable => "variable",
        SymbolKind::DefaultExport => "default export",
        SymbolKind::Other => "export",
    }
}

fn non_empty(value: &String) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

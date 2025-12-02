//! Main MDX to JSX conversion entry point

use anyhow::{Context, Result, anyhow};
use markdown::mdast::Node;

use super::context::CodegenContext;
use crate::frontmatter::extract_frontmatter;

/// Convert MDX mdast to JSX string with React 19 and MDX v3 compatibility
///
/// This is the main entry point for MDX compilation. It takes a markdown AST
/// (from markdown-rs) and converts it to JSX code that:
///
/// - Integrates with MDXProvider for component customization
/// - Spreads props on all HTML elements for styling/customization
/// - Uses stable keys for list items and table rows
/// - Handles ESM imports/exports properly
/// - Supports GFM, footnotes, and math
pub fn mdast_to_jsx(root: &Node) -> Result<String> {
    mdast_to_jsx_with_options(root, &crate::MdxOptions::default())
}

/// Convert MDX mdast to JSX string with custom options
///
/// This function is identical to `mdast_to_jsx()` but allows passing custom
/// options including plugins for AST and JSX transformation.
///
/// # Plugin Execution Order
///
/// 1. Extract frontmatter from AST (or use pre-extracted from options)
/// 2. Run all `plugin.transform_ast()` in registration order
/// 3. Convert AST to JSX
/// 4. Run all `plugin.transform_jsx()` in registration order
pub fn mdast_to_jsx_with_options(root: &Node, options: &crate::MdxOptions) -> Result<String> {
    // Use pre-extracted frontmatter if provided, otherwise extract from AST
    let (mut cleaned_root, frontmatter) = if options.frontmatter.is_some() {
        // Frontmatter already extracted, just clone root and use provided frontmatter
        (root.clone(), options.frontmatter.clone())
    } else {
        // Extract frontmatter from AST
        extract_frontmatter(root)?
    };

    // Run AST transformation plugins
    for plugin in &options.plugins {
        tracing::debug!(plugin = plugin.name(), "Running AST transformation plugin");
        plugin.transform_ast(&mut cleaned_root).with_context(|| {
            format!(
                "Plugin '{}' failed during AST transformation",
                plugin.name()
            )
        })?;
    }

    let mut imports = Vec::new();
    let mut named_exports = Vec::new();
    let mut reexports = Vec::new();
    let mut jsx_elements = Vec::new();
    let mut ctx = CodegenContext::new();

    // NOTE: For remote MDX, we don't import useMDXComponents here
    // because MDXRemote handles all component resolution.
    // These imports would cause Server Component boundary issues.

    // imports.push(
    //     "import {useMDXComponents as _provideComponents} from '@fob/mdx-runtime';".to_string(),
    // );
    // imports.push(
    //     "import {useTaskListContext as _useTaskListContext} from '@fob/mdx-runtime';".to_string(),
    // );

    if let Node::Root(root_node) = &cleaned_root {
        for child in &root_node.children {
            match child {
                Node::MdxjsEsm(esm) => {
                    // Categorize ESM statements
                    let code = esm.value.trim();

                    if crate::esm::is_reexport(code) {
                        // Re-exports: export {...} from './x'
                        reexports.push(code.to_string());
                    } else if crate::esm::has_named_exports(code) {
                        // Named exports: export const meta = ...
                        named_exports.push(code.to_string());
                    } else if !code.starts_with("export default") {
                        // Regular imports: import {...} from './x'
                        // Note: markdown-rs may combine multiple imports into one ESM node
                        imports.push(code.to_string());

                        // Extract imported component names from each line
                        // (markdown-rs combines consecutive imports into one node)
                        for line in code.lines() {
                            let line = line.trim();
                            if line.starts_with("import ") {
                                let imported_names = crate::esm::extract_imported_names(line);

                                // Debug logging to diagnose MDX import issues
                                tracing::info!(
                                    import_statement = line,
                                    extracted_names = ?imported_names,
                                    "Extracted component names from MDX import"
                                );

                                ctx.imported_components.extend(imported_names);
                            }
                        }
                    }
                    // Skip export default statements - we generate our own
                }
                _ => {
                    // Convert markdown/MDX nodes to JSX - use full path to avoid circular dependency
                    if let Some(jsx_value) =
                        super::super::nodes::node_to_jsx(child, &mut ctx, false)?
                    {
                        jsx_elements.push(jsx_value.to_js()); // Convert JsValue to String
                    }
                }
            }
        }
    } else {
        return Err(anyhow!("Expected Root node, got {:?}", cleaned_root));
    }

    // Add frontmatter export if present
    // Also extract prop names for MDXContent signature
    let prop_names: Vec<String> = frontmatter
        .as_ref()
        .map(|fm| fm.prop_names().into_iter().map(String::from).collect())
        .unwrap_or_default();

    if let Some(ref fm) = frontmatter {
        // Serialize frontmatter as JSON and inject as a named export
        let json_str = serde_json::to_string(&fm.data)
            .with_context(|| "Failed to serialize frontmatter to JSON")?;

        named_exports.push(format!("export const frontmatter = {};", json_str));

        // Export propDefinitions if props exist
        if !fm.props.is_empty() {
            let props_json = serde_json::to_string(&fm.props)
                .with_context(|| "Failed to serialize propDefinitions to JSON")?;
            named_exports.push(format!("export const propDefinitions = {};", props_json));
        }
    }

    // Generate MDXContent component with React 19 JSX runtime
    let (content, needs_fragment) = if jsx_elements.is_empty() {
        (String::from("null"), false)
    } else if jsx_elements.len() == 1 {
        (jsx_elements[0].clone(), false)
    } else {
        // Use jsxs for static multi-child Fragments
        // jsxs tells React: "these children are static, skip key warnings"
        (
            format!(
                "_jsxs(_Fragment, {{children: [{}]}})",
                jsx_elements.join(", ")
            ),
            true,
        )
    };

    // Build data props destructuring if props exist
    let data_props_destructure = if prop_names.is_empty() {
        String::new()
    } else {
        format!("\n  const {{ {} }} = _dataProps;", prop_names.join(", "))
    };

    // Build MDXContent function body (shared between formats)
    let mdx_content_body = format!(
        r#"function MDXContent({{components: _cProp = {{}}, _dataProps = {{}}, ...props}}) {{{data_props}
  const _components = Object.assign({{
    h1: "h1", h2: "h2", h3: "h3", h4: "h4", h5: "h5", h6: "h6",
    p: "p", a: "a", strong: "strong", em: "em", code: "code", pre: "pre",
    blockquote: "blockquote", ul: "ul", ol: "ol", li: "li",
    table: "table", thead: "thead", tbody: "tbody", tr: "tr", th: "th", td: "td",
    hr: "hr", br: "br", img: "img", del: "del", div: "div", span: "span", sup: "sup", input: "input"
  }}, _cProp);
  const _taskListCtx = null; // Task list context disabled for now
  const _handleTaskToggle = (e) => {{
    const taskId = e.target.getAttribute('data-task-id');
    if (taskId && _taskListCtx) {{
      _taskListCtx.toggleTask(taskId, e.target.checked);
    }}
  }};
  return {content};
}}"#,
        data_props = data_props_destructure,
        content = content
    );

    // Build final output based on format
    let mut output = String::new();

    match options.output_format {
        crate::OutputFormat::Program => {
            // Program format: ES module with import/export
            // Add JSX runtime imports based on what we need
            let jsx_runtime = if needs_fragment {
                format!(
                    "import {{jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment}} from '{}';",
                    options.jsx_runtime
                )
            } else {
                format!(
                    "import {{jsx as _jsx, jsxs as _jsxs}} from '{}';",
                    options.jsx_runtime
                )
            };
            imports.insert(0, jsx_runtime);

            // Add imports
            if !imports.is_empty() {
                output.push_str(&imports.join("\n"));
                output.push_str("\n\n");
            }

            // Add named exports before default export
            if !named_exports.is_empty() {
                output.push_str(&named_exports.join("\n"));
                output.push_str("\n\n");
            }

            // Add MDXContent default export
            output.push_str("export default ");
            output.push_str(&mdx_content_body);

            // Add re-exports after default export
            if !reexports.is_empty() {
                output.push_str("\n\n");
                output.push_str(&reexports.join("\n"));
            }
        }
        crate::OutputFormat::FunctionBody => {
            // Function-body format: for runtime eval with new Function()
            output.push_str("\"use strict\";\n");

            // Provide JSX runtime from arguments[0]
            if needs_fragment {
                output.push_str(
                    "const {jsx: _jsx, jsxs: _jsxs, Fragment: _Fragment} = arguments[0];\n",
                );
            } else {
                output.push_str("const {jsx: _jsx, jsxs: _jsxs} = arguments[0];\n");
            }

            // Extract named export names for return object
            let mut export_names = Vec::new();

            // Add named exports as const declarations (remove "export" keyword)
            for export in &named_exports {
                // Remove "export " prefix
                let const_decl = export.replace("export ", "");
                output.push_str(&const_decl);
                output.push('\n');

                // Extract variable name for return object
                // Pattern: "const NAME = ..." or "const NAME ="
                if let Some(const_start) = const_decl.find("const ") {
                    let after_const = &const_decl[const_start + 6..];
                    if let Some(name_end) = after_const.find([' ', '=']) {
                        let name = after_const[..name_end].trim();
                        if !name.is_empty() {
                            export_names.push(name.to_string());
                        }
                    }
                }
            }

            // Add MDXContent function (without export default)
            output.push_str(&mdx_content_body);
            output.push('\n');

            // Build return object with all exports
            output.push_str("return {default: MDXContent");
            // Add named exports to return object
            for name in &export_names {
                output.push_str(&format!(", {}: {}", name, name));
            }
            output.push_str("};\n");
        }
    }

    // Run JSX transformation plugins
    for plugin in &options.plugins {
        tracing::debug!(plugin = plugin.name(), "Running JSX transformation plugin");
        plugin.transform_jsx(&mut output).with_context(|| {
            format!(
                "Plugin '{}' failed during JSX transformation",
                plugin.name()
            )
        })?;
    }

    Ok(output)
}

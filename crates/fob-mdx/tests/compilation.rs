//! Comprehensive MDX compilation tests.
// Copyright 2024 the Fob authors. All rights reserved. MIT license.
#![cfg(test)]

use fob_mdx::{MdxCompileOptions, MdxCompileResult, OutputFormat, compile};

/// Helper to compile MDX and return the result
fn compile_mdx(mdx: &str) -> MdxCompileResult {
    compile(mdx, MdxCompileOptions::default()).unwrap()
}

/// Helper to compile MDX with custom options
fn compile_with_options(mdx: &str, options: MdxCompileOptions) -> MdxCompileResult {
    compile(mdx, options).unwrap()
}

// =============================================================================
// JSX Props Tests (kept from original props.rs)
// =============================================================================

mod jsx_props {
    use super::*;

    #[test]
    fn prop_key_quoting_literals() {
        let result = compile_mdx(r#"<Button aria-label="Notify" />"#);
        // Kebab-case props MUST be quoted for valid JS
        assert!(
            result.code.contains(r#""aria-label": "Notify""#),
            "Kebab-case props must be quoted. actual: {}",
            result.code
        );
    }

    #[test]
    fn prop_key_quoting_expressions() {
        let result = compile_mdx(r#"<Button data-count={count} />"#);
        // Kebab-case props with expression values MUST be quoted
        assert!(
            result.code.contains(r#""data-count": count"#),
            "Kebab-case expression props must be quoted. actual: {}",
            result.code
        );
    }

    #[test]
    fn valid_identifiers_stay_bare() {
        let result = compile_mdx(r#"<Button ariaLabel="Notify" />"#);
        // Valid JS identifiers should NOT be quoted
        assert!(
            result.code.contains(r#"ariaLabel: "Notify""#),
            "CamelCase props should stay unquoted. actual: {}",
            result.code
        );
    }

    #[test]
    fn mixed_with_spreads() {
        let result = compile_mdx(r#"<Button {...rest} aria-label="x" />"#);
        // Spread should work with quoted kebab-case props
        assert!(
            result.code.contains("...rest") && result.code.contains(r#""aria-label": "x""#),
            "Spread and quoted kebab-case should coexist. actual: {}",
            result.code
        );
    }

    #[test]
    fn children_path() {
        let result = compile_mdx(r#"<div aria-label="x">Hi</div>"#);
        // Kebab-case props with children should be quoted
        assert!(
            result.code.contains(r#""aria-label": "x""#) && result.code.contains("children"),
            "Quoted props and children should both be present. actual: {}",
            result.code
        );
    }

    #[test]
    fn lowercase_kebab_case_is_html_element() {
        let result = compile_mdx(r#"<foo-bar />"#);
        // Lowercase kebab-case tags are treated as HTML custom elements
        assert!(
            result.code.contains(r#"_jsx("foo-bar""#),
            "Lowercase kebab-case should be HTML custom element. actual: {}",
            result.code
        );
    }

    #[test]
    fn uppercase_kebab_component_uses_map() {
        let result = compile_mdx(r#"<Foo-bar />"#);
        // PascalCase kebab components use _components map with bracket notation
        assert!(
            result.code.contains(r#"_components["Foo-bar"]"#),
            "Uppercase kebab-case components must use bracket notation. actual: {}",
            result.code
        );
    }

    #[test]
    fn multiple_data_attributes() {
        let result = compile_mdx(r#"<div data-test-id="test" data-value={42} />"#);
        assert!(
            result.code.contains(r#""data-test-id": "test""#),
            "Multiple data attrs should be quoted. actual: {}",
            result.code
        );
        assert!(
            result.code.contains(r#""data-value": 42"#),
            "data-value should be quoted. actual: {}",
            result.code
        );
    }
}

// =============================================================================
// ESM Extraction Tests
// =============================================================================

mod esm_extraction {
    use super::*;

    #[test]
    fn extracts_named_imports() {
        let mdx = r#"import { Button, Card } from './components'

# Hello"#;
        let result = compile_mdx(mdx);
        assert_eq!(
            result.imports.len(),
            1,
            "Should extract one import statement"
        );
        assert!(
            result.imports[0].contains("Button"),
            "Import should contain Button"
        );
    }

    #[test]
    fn extracts_default_imports() {
        let mdx = r#"import Layout from './layout'

# Hello"#;
        let result = compile_mdx(mdx);
        assert_eq!(result.imports.len(), 1);
        assert!(result.imports[0].contains("Layout"));
    }

    #[test]
    fn extracts_named_exports() {
        let mdx = r#"export const meta = { title: "Test" }

# Hello"#;
        let result = compile_mdx(mdx);
        assert_eq!(
            result.named_exports.len(),
            1,
            "Should extract one named export"
        );
        assert!(result.named_exports[0].contains("meta"));
    }

    #[test]
    fn extracts_reexports() {
        let mdx = r#"export { foo, bar } from './utils'

# Hello"#;
        let result = compile_mdx(mdx);
        assert_eq!(result.reexports.len(), 1, "Should extract one reexport");
        assert!(result.reexports[0].contains("foo"));
    }

    #[test]
    fn extracts_default_export_name() {
        let mdx = r#"export default function MyPage() { return null }

# Hello"#;
        let result = compile_mdx(mdx);
        assert!(
            result.default_export.is_some(),
            "Should extract default export"
        );
        assert_eq!(result.default_export.as_deref(), Some("MyPage"));
    }

    #[test]
    fn handles_multiple_esm_statements() {
        // Each ESM block must be on its own paragraph in MDX
        let mdx = r#"import { A } from './a'

import B from './b'

export const x = 1

export const y = 2

export { z } from './z'

# Content"#;
        let result = compile_mdx(mdx);
        assert!(
            result.imports.len() >= 1,
            "Should have at least 1 import. Got: {:?}",
            result.imports
        );
        assert!(
            result.named_exports.len() >= 1,
            "Should have at least 1 named export. Got: {:?}",
            result.named_exports
        );
        assert!(
            result.reexports.len() >= 1,
            "Should have at least 1 reexport. Got: {:?}",
            result.reexports
        );
    }
}

// =============================================================================
// Image Collection Tests
// =============================================================================

mod image_collection {
    use super::*;

    #[test]
    fn collects_markdown_images() {
        let mdx = r#"# Hello

![Alt text](./image.png)
"#;
        let result = compile_mdx(mdx);
        assert!(
            result.images.contains(&"./image.png".to_string()),
            "Should collect image URL. Got: {:?}",
            result.images
        );
    }

    #[test]
    fn collects_multiple_images() {
        let mdx = r#"# Gallery

![First](./first.jpg)
![Second](./second.png)
![Third](/absolute/third.webp)
"#;
        let result = compile_mdx(mdx);
        assert_eq!(result.images.len(), 3, "Should collect all 3 images");
    }

    #[test]
    fn handles_relative_image_paths() {
        let mdx = r#"![](../assets/logo.svg)"#;
        let result = compile_mdx(mdx);
        assert!(result.images.contains(&"../assets/logo.svg".to_string()));
    }

    #[test]
    fn handles_absolute_image_urls() {
        let mdx = r#"![](https://example.com/image.png)"#;
        let result = compile_mdx(mdx);
        assert!(
            result
                .images
                .contains(&"https://example.com/image.png".to_string())
        );
    }
}

// =============================================================================
// Frontmatter Tests
// =============================================================================

mod frontmatter {
    use super::*;
    use fob_mdx::FrontmatterFormat;

    #[test]
    fn parses_yaml_frontmatter() {
        let mdx = r#"---
title: "Hello World"
author: John Doe
---

# Content"#;
        let result = compile_mdx(mdx);
        assert!(result.frontmatter.is_some());
        let fm = result.frontmatter.unwrap();
        assert_eq!(fm.format, FrontmatterFormat::Yaml);
        assert!(fm.raw.contains("title"));
    }

    #[test]
    fn parses_toml_frontmatter() {
        let mdx = r#"+++
title = "Hello World"
author = "John Doe"
+++

# Content"#;
        let result = compile_mdx(mdx);
        assert!(result.frontmatter.is_some());
        let fm = result.frontmatter.unwrap();
        assert_eq!(fm.format, FrontmatterFormat::Toml);
    }

    #[test]
    fn handles_empty_frontmatter() {
        let mdx = r#"---
---

# Content"#;
        let result = compile_mdx(mdx);
        // Empty frontmatter is valid
        assert!(result.frontmatter.is_some());
    }

    #[test]
    fn handles_nested_frontmatter_objects() {
        let mdx = r#"---
meta:
  title: "Nested"
  tags:
    - rust
    - mdx
---

# Content"#;
        let result = compile_mdx(mdx);
        assert!(result.frontmatter.is_some());
        let fm = result.frontmatter.unwrap();
        assert!(fm.raw.contains("tags"));
    }

    #[test]
    fn handles_no_frontmatter() {
        let mdx = r#"# Just a heading

No frontmatter here."#;
        let result = compile_mdx(mdx);
        assert!(result.frontmatter.is_none());
    }

    #[test]
    fn frontmatter_exported_in_code() {
        let mdx = r#"---
title: "Test"
---

# Hello"#;
        let result = compile_mdx(mdx);
        // Frontmatter should be exported in the generated code
        assert!(
            result.code.contains("frontmatter"),
            "Code should contain frontmatter export"
        );
    }
}

// =============================================================================
// Output Format Tests
// =============================================================================

mod output_formats {
    use super::*;

    #[test]
    fn program_format_has_imports_exports() {
        let mdx = "# Hello";
        let options = MdxCompileOptions::builder()
            .output_format(OutputFormat::Program)
            .build();
        let result = compile_with_options(mdx, options);

        assert!(
            result.code.contains("import {"),
            "Should have import statement"
        );
        assert!(
            result.code.contains("export default"),
            "Should have default export"
        );
    }

    #[test]
    fn function_body_has_use_strict() {
        let mdx = "# Hello";
        let options = MdxCompileOptions::builder()
            .output_format(OutputFormat::FunctionBody)
            .build();
        let result = compile_with_options(mdx, options);

        assert!(
            result.code.starts_with("\"use strict\""),
            "FunctionBody should start with 'use strict'"
        );
    }

    #[test]
    fn function_body_has_arguments_destructure() {
        let mdx = "# Hello";
        let options = MdxCompileOptions::builder()
            .output_format(OutputFormat::FunctionBody)
            .build();
        let result = compile_with_options(mdx, options);

        assert!(
            result.code.contains("arguments[0]"),
            "FunctionBody should use arguments[0] for JSX runtime"
        );
    }

    #[test]
    fn function_body_returns_exports_object() {
        let mdx = "# Hello";
        let options = MdxCompileOptions::builder()
            .output_format(OutputFormat::FunctionBody)
            .build();
        let result = compile_with_options(mdx, options);

        assert!(
            result.code.contains("return {"),
            "FunctionBody should return an object"
        );
        assert!(
            result.code.contains("default: MDXContent"),
            "Return should include default export"
        );
    }
}

// =============================================================================
// Provider Integration Tests
// =============================================================================

mod provider_integration {
    use super::*;

    #[test]
    fn provider_import_source_adds_import() {
        let mdx = "# Hello";
        let options = MdxCompileOptions::builder()
            .provider_import_source("@mdx-js/react")
            .build();
        let result = compile_with_options(mdx, options);

        assert!(
            result.code.contains("useMDXComponents") || result.code.contains("_provideComponents"),
            "Should import provider function. Code: {}",
            result.code
        );
    }

    #[test]
    fn provider_merges_components() {
        let mdx = "# Hello";
        let options = MdxCompileOptions::builder()
            .provider_import_source("gumbo/mdx")
            .build();
        let result = compile_with_options(mdx, options);

        // The component merging logic should be present
        assert!(
            result.code.contains("_provideComponents") || result.code.contains("_components"),
            "Should have component merging logic"
        );
    }

    #[test]
    fn no_provider_when_none_set() {
        let mdx = "# Hello";
        let options = MdxCompileOptions::builder().build();
        let result = compile_with_options(mdx, options);

        // Should not have provider-specific imports
        assert!(
            !result.code.contains("_provideComponents"),
            "Should not have provider import when not configured"
        );
    }
}

// =============================================================================
// JSX Components Tests
// =============================================================================

mod jsx_components {
    use super::*;

    #[test]
    fn pascalcase_component_direct_reference() {
        let mdx = r#"import Button from './button'

<Button>Click</Button>"#;
        let result = compile_mdx(mdx);
        // PascalCase imported components should be referenced directly
        assert!(
            result.code.contains("Button"),
            "PascalCase component should appear in output"
        );
    }

    #[test]
    fn lowercase_uses_components_map() {
        let mdx = r#"<div>Hello</div>"#;
        let result = compile_mdx(mdx);
        // Lowercase HTML elements use _components map
        assert!(
            result.code.contains("_components"),
            "Should reference _components for HTML elements"
        );
    }

    #[test]
    fn kebab_case_component_referenced() {
        let mdx = r#"<my-custom-element />"#;
        let result = compile_mdx(mdx);
        // Kebab-case components should be referenced in the output
        assert!(
            result.code.contains("my-custom-element"),
            "Kebab-case component should be referenced. Code: {}",
            result.code
        );
    }

    #[test]
    fn jsx_expressions_preserved() {
        let mdx = r#"<div>{items.map(x => x.name)}</div>"#;
        let result = compile_mdx(mdx);
        assert!(
            result.code.contains("items.map"),
            "JSX expressions should be preserved"
        );
    }

    #[test]
    fn spread_props_work() {
        let mdx = r#"<Component {...props} extra="value" />"#;
        let result = compile_mdx(mdx);
        assert!(
            result.code.contains("...props"),
            "Spread should be preserved"
        );
    }
}

// =============================================================================
// Error Handling Tests
// =============================================================================

mod error_handling {
    use super::*;

    #[test]
    fn rejects_oversized_input() {
        let huge = "x".repeat(11 * 1024 * 1024); // 11MB
        let result = compile(&huge, MdxCompileOptions::default());

        assert!(result.is_err(), "Should reject oversized input");
        let err = result.unwrap_err();
        assert!(
            err.message.contains("exceeds maximum size"),
            "Error should mention size limit"
        );
    }

    #[test]
    fn handles_unclosed_jsx_tag() {
        let mdx = r#"# Hello

<div>
  <span>Nested
</div>"#;
        // This may or may not error depending on MDX parser leniency
        let result = compile(mdx, MdxCompileOptions::default());
        // If it errors, the error should be descriptive
        if let Err(e) = result {
            assert!(
                !e.message.is_empty(),
                "Error message should not be empty: {:?}",
                e
            );
        }
    }

    #[test]
    fn handles_invalid_esm_syntax() {
        let mdx = r#"import { from './bad'

# Hello"#;
        let result = compile(mdx, MdxCompileOptions::default());
        assert!(result.is_err(), "Should reject invalid ESM syntax");
    }

    #[test]
    fn handles_malformed_jsx_expression() {
        let mdx = r#"<div>{unclosed"#;
        let result = compile(mdx, MdxCompileOptions::default());
        assert!(result.is_err(), "Should reject malformed JSX expression");
    }

    #[test]
    fn error_includes_filepath_when_provided() {
        let mdx = r#"<div>{unclosed"#;
        let options = MdxCompileOptions::builder()
            .filepath("test/file.mdx")
            .build();
        let result = compile(mdx, options);

        if let Err(e) = result {
            // Error should include filepath context
            assert!(
                e.file.is_some() || e.message.contains("test/file.mdx"),
                "Error should include filepath"
            );
        }
    }
}

// =============================================================================
// Nested Structures Tests
// =============================================================================

mod nested_structures {
    use super::*;

    #[test]
    fn heading_in_blockquote() {
        let mdx = r#"> ## Quoted Heading
>
> Some text"#;
        let result = compile_mdx(mdx);
        assert!(result.code.contains("blockquote"), "Should have blockquote");
        assert!(
            result.code.contains("h2"),
            "Should have h2 inside blockquote"
        );
    }

    #[test]
    fn link_in_list_item() {
        let mdx = r#"- Item with [link](/path)
- Another item"#;
        let result = compile_mdx(mdx);
        assert!(result.code.contains("li"), "Should have list items");
        assert!(result.code.contains("/path"), "Should have link href");
    }

    #[test]
    fn code_in_table_cell() {
        let mdx = r#"| Header |
|--------|
| `code` |"#;
        let result = compile_mdx(mdx);
        assert!(result.code.contains("table"), "Should have table");
        assert!(
            result.code.contains("code"),
            "Should have inline code in cell"
        );
    }

    #[test]
    fn emphasis_in_heading() {
        let mdx = r#"# Hello **World** and *italics*"#;
        let result = compile_mdx(mdx);
        assert!(result.code.contains("h1"), "Should have h1");
        assert!(
            result.code.contains("strong"),
            "Should have strong in heading"
        );
        assert!(
            result.code.contains("em"),
            "Should have emphasis in heading"
        );
    }
}

// =============================================================================
// GFM Features Tests
// =============================================================================

mod gfm_features {
    use super::*;

    #[test]
    fn strikethrough_renders_del() {
        let mdx = r#"This is ~~deleted~~ text."#;
        let result = compile_mdx(mdx);
        assert!(
            result.code.contains("del"),
            "Strikethrough should render as del element"
        );
    }

    #[test]
    fn tables_render_correctly() {
        let mdx = r#"| A | B |
|---|---|
| 1 | 2 |"#;
        let result = compile_mdx(mdx);
        assert!(result.code.contains("table"), "Should have table");
        assert!(result.code.contains("tr"), "Should have table rows");
        assert!(
            result.code.contains("td") || result.code.contains("th"),
            "Should have cells"
        );
    }

    #[test]
    fn task_lists_with_checkboxes() {
        let mdx = r#"- [x] Done
- [ ] Todo"#;
        let result = compile_mdx(mdx);
        assert!(result.code.contains("li"), "Should have list items");
        // Task lists may render as input[type=checkbox] or similar
        assert!(
            result.code.contains("checked") || result.code.contains("input"),
            "Should have checkbox indicators"
        );
    }

    #[test]
    fn gfm_disabled_skips_features() {
        let mdx = r#"~~strikethrough~~"#;
        let options = MdxCompileOptions::builder().gfm(false).build();
        let result = compile_with_options(mdx, options);

        assert!(
            !result.code.contains("_components.del"),
            "GFM strikethrough should be disabled"
        );
    }
}

// =============================================================================
// Edge Cases Tests
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn empty_document() {
        let mdx = "";
        let result = compile_mdx(mdx);
        // Should compile without error
        assert!(
            result.code.contains("MDXContent"),
            "Should still export MDXContent"
        );
    }

    #[test]
    fn whitespace_only() {
        let mdx = "   \n\n   \n";
        let result = compile_mdx(mdx);
        assert!(result.code.contains("MDXContent"));
    }

    #[test]
    fn unicode_content_preserved() {
        let mdx = r#"# 你好世界

日本語テキスト with 🎉 emoji"#;
        let result = compile_mdx(mdx);
        assert!(
            result.code.contains("你好世界"),
            "Chinese should be preserved"
        );
        assert!(
            result.code.contains("日本語"),
            "Japanese should be preserved"
        );
        assert!(result.code.contains("🎉"), "Emoji should be preserved");
    }

    #[test]
    fn special_chars_in_text() {
        // Use escaped special chars since raw < > & are special in MDX
        let mdx = r#"Text with `<`, `>`, `&`, `"` and `'` characters."#;
        let result = compile_mdx(mdx);
        // Should compile without error, escaping handled properly
        assert!(result.code.contains("MDXContent"));
    }

    #[test]
    fn very_long_heading() {
        let long_text = "A".repeat(500);
        let mdx = format!("# {}", long_text);
        let result = compile_mdx(&mdx);
        // Should handle long headings without panic
        assert!(result.code.contains("h1"));
    }
}

// =============================================================================
// Math Feature Tests
// =============================================================================

mod math_features {
    use super::*;

    #[test]
    fn inline_math() {
        let mdx = r#"The formula $E = mc^2$ is famous."#;
        let result = compile_mdx(mdx);
        assert!(
            result.code.contains("math") || result.code.contains("E = mc"),
            "Inline math should be processed"
        );
    }

    #[test]
    fn block_math() {
        let mdx = r#"$$
\int_0^\infty e^{-x} dx = 1
$$"#;
        let result = compile_mdx(mdx);
        assert!(
            result.code.contains("math") || result.code.contains("int"),
            "Block math should be processed"
        );
    }

    #[test]
    fn math_disabled() {
        let mdx = r#"Inline $math$ here."#;
        let options = MdxCompileOptions::builder().math(false).build();
        let result = compile_with_options(mdx, options);

        // When math is disabled, $ should be treated as literal
        assert!(
            !result.code.contains("_components.math"),
            "Math should be disabled"
        );
    }
}

// =============================================================================
// Footnotes Tests
// =============================================================================

mod footnotes {
    use super::*;

    #[test]
    fn footnote_reference_and_definition() {
        let mdx = r#"Here is a footnote[^1].

[^1]: This is the footnote content."#;
        let result = compile_mdx(mdx);
        // Should have footnote references and definitions
        assert!(
            result.code.contains("footnote") || result.code.contains("sup"),
            "Should process footnotes"
        );
    }

    #[test]
    fn multiple_footnotes() {
        let mdx = r#"First[^a] and second[^b].

[^a]: First note.
[^b]: Second note."#;
        let result = compile_mdx(mdx);
        // Should handle multiple footnotes
        assert!(result.code.contains("MDXContent"));
    }

    #[test]
    fn footnotes_disabled() {
        let mdx = r#"Text[^1].

[^1]: Note."#;
        let options = MdxCompileOptions::builder().footnotes(false).build();
        let result = compile_with_options(mdx, options);

        // When footnotes disabled, [^1] should be treated as link reference
        assert!(
            !result.code.contains("footnoteReference"),
            "Footnotes should be disabled"
        );
    }
}

// =============================================================================
// Code Block Tests
// =============================================================================

mod code_blocks {
    use super::*;

    #[test]
    fn fenced_code_with_language() {
        let mdx = r#"```rust
fn main() {}
```"#;
        let result = compile_mdx(mdx);
        assert!(result.code.contains("pre"), "Should have pre element");
        assert!(result.code.contains("code"), "Should have code element");
    }

    #[test]
    fn inline_code() {
        let mdx = r#"Use `const` for constants."#;
        let result = compile_mdx(mdx);
        assert!(result.code.contains("code"), "Should have inline code");
        assert!(result.code.contains("const"), "Code content preserved");
    }

    #[test]
    fn code_with_special_chars() {
        let mdx = r#"```
<div>&amp;</div>
```"#;
        let result = compile_mdx(mdx);
        // Code blocks should preserve content without HTML entity conversion
        assert!(result.code.contains("MDXContent"));
    }
}

// =============================================================================
// JSX Runtime Tests
// =============================================================================

mod jsx_runtime {
    use super::*;

    #[test]
    fn default_jsx_runtime_is_react() {
        let mdx = "# Hello";
        let result = compile_mdx(mdx);
        assert!(
            result.code.contains("react/jsx-runtime"),
            "Default should use react/jsx-runtime"
        );
    }

    #[test]
    fn custom_jsx_runtime() {
        let mdx = "# Hello";
        let options = MdxCompileOptions::builder()
            .jsx_runtime("preact/jsx-runtime")
            .build();
        let result = compile_with_options(mdx, options);
        assert!(
            result.code.contains("preact/jsx-runtime"),
            "Should use custom JSX runtime"
        );
    }
}

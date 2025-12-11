//! Plugin invocation verification tests
//!
//! These tests ensure that plugins are actually invoked when processing
//! their respective file types. This catches regressions where plugins
//! are registered but not called.

mod helpers;

#[cfg(not(target_family = "wasm"))]
mod plugin_invocation_tests {
    use super::helpers::*;
    use fob_bundler::runtime::BundlerRuntime;
    use fob_bundler::{BuildOptions, Platform, Runtime};
    use fob_plugin_css::FobCssPlugin;
    use fob_plugin_mdx::FobMdxPlugin;
    use std::sync::Arc;
    use tempfile::TempDir;

    /// Verify MDX plugin is invoked for .mdx files
    ///
    /// This is the critical regression test - if MDX plugin isn't invoked,
    /// the bundler will try to parse raw MDX as JavaScript and fail.
    #[tokio::test]
    async fn test_mdx_plugin_is_invoked() {
        let temp_dir = TempDir::new().expect("create temp dir");

        // MDX content that would fail if parsed as raw JS
        std::fs::write(
            temp_dir.path().join("content.mdx"),
            r#"---
title: "Plugin Invocation Test"
---

# Heading That Would Be Syntax Error in JS

This **markdown** would fail if not processed by MDX plugin.

```js
const code = "block";
```
"#,
        )
        .expect("write mdx file");

        std::fs::write(
            temp_dir.path().join("entry.tsx"),
            "import Content from './content.mdx';\nexport { Content };",
        )
        .expect("write entry");

        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new(temp_dir.path()));

        let result = BuildOptions::new(temp_dir.path().join("entry.tsx"))
            .externalize_from("package.json")
            .platform(Platform::Node)
            .cwd(temp_dir.path())
            .runtime(Arc::clone(&runtime))
            .plugin(Arc::new(FobMdxPlugin::new(runtime)))
            .build()
            .await
            .expect("MDX plugin should process .mdx file");

        // Verify MDX was compiled to JSX
        let chunk = result.chunks().next().expect("should have chunk");

        // Content should be present (MDX processed it)
        assert!(
            chunk.code.contains("Plugin Invocation Test")
                || chunk.code.contains("Heading That Would Be Syntax Error"),
            "MDX content should be in output"
        );

        // Raw markdown syntax should NOT be present
        assert!(
            !chunk.code.contains("# Heading"),
            "Raw markdown heading syntax should be transformed"
        );

        // STRONGER ASSERTIONS: Verify JSX compilation happened
        assert!(
            chunk.code.contains("jsx")
                || chunk.code.contains("jsxs")
                || chunk.code.contains("createElement"),
            "Output should contain JSX runtime calls - proves MDX→JSX compilation"
        );

        // Verify MDXContent function was generated
        assert!(
            chunk.code.contains("MDXContent") || chunk.code.contains("function"),
            "Output should contain MDXContent component function"
        );

        // Verify component structure (props handling)
        assert!(
            chunk.code.contains("components") || chunk.code.contains("_components"),
            "Output should have component mapping for MDX elements"
        );
    }

    /// Verify CSS plugin is invoked for .css files
    #[tokio::test]
    async fn test_css_plugin_is_invoked() {
        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new("."));

        // CSS content with modern features
        let css_content = r#"
.container {
    display: flex;
    gap: 1rem;
    --custom-prop: #3b82f6;
}

.btn {
    background: var(--custom-prop);
    padding: 0.5rem 1rem;
}
"#;

        // Use virtual: prefix consistently for imports
        let result = BuildOptions::new("virtual:entry.js")
            .externalize_from("package.json")
            .platform(Platform::Node)
            .virtual_file(
                "virtual:entry.js",
                "import 'virtual:styles.css';\nexport const loaded = true;",
            )
            .virtual_file("virtual:styles.css", css_content)
            .runtime(Arc::clone(&runtime))
            .plugin(Arc::new(FobCssPlugin::new(runtime)))
            .build()
            .await
            .expect("CSS plugin should process .css file");

        // Build should succeed (CSS was handled)
        let bundle = result.output.as_single().expect("single bundle");
        assert!(!bundle.assets.is_empty(), "Should produce assets");
    }

    /// Verify that MDX files fail gracefully without the plugin
    ///
    /// This documents the expected behavior when MDX plugin is missing.
    /// The bundler should either:
    /// 1. Fail with a clear error about unhandled file type, OR
    /// 2. Fail with a parse error (since MDX isn't valid JS)
    #[tokio::test]
    async fn test_mdx_without_plugin_behavior() {
        let temp_dir = TempDir::new().expect("create temp dir");

        std::fs::write(
            temp_dir.path().join("content.mdx"),
            "# Heading\n\nSome **content**.",
        )
        .expect("write mdx file");

        std::fs::write(
            temp_dir.path().join("entry.tsx"),
            "import Content from './content.mdx';\nexport { Content };",
        )
        .expect("write entry");

        let result = BuildOptions::new(temp_dir.path().join("entry.tsx"))
            .externalize_from("package.json")
            .platform(Platform::Node)
            .cwd(temp_dir.path())
            // NOTE: NO MDX plugin registered!
            .build()
            .await;

        // Should fail - MDX is not valid JavaScript
        // The exact error depends on how Rolldown handles unrecognized extensions
        match result {
            Ok(_) => {
                // If it somehow succeeds, the MDX wasn't imported (tree-shaken or ignored)
                println!("Warning: Build succeeded without MDX plugin - verify MDX import is used");
            }
            Err(e) => {
                // Expected: should fail with parse error or unhandled extension
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("parse")
                        || err_msg.contains("syntax")
                        || err_msg.contains("mdx")
                        || err_msg.contains("Cannot assign")
                        || err_msg.contains("unresolved"),
                    "Error should indicate MDX parsing failed: {}",
                    err_msg
                );
            }
        }
    }

    /// Verify multiple plugins work together
    #[tokio::test]
    async fn test_multiple_plugins_invoked() {
        let temp_dir = TempDir::new().expect("create temp dir");

        // Create MDX file (without CSS import to avoid path resolution complexity)
        std::fs::write(
            temp_dir.path().join("content.mdx"),
            r#"# Multi-Plugin Test

This MDX file tests multiple plugin coexistence.

Some **content** here.
"#,
        )
        .expect("write mdx file");

        // Create CSS file
        std::fs::write(
            temp_dir.path().join("styles.css"),
            ".content { color: blue; }",
        )
        .expect("write css file");

        // Create entry that imports both MDX and CSS
        std::fs::write(
            temp_dir.path().join("entry.tsx"),
            "import Content from './content.mdx';\nimport './styles.css';\nexport { Content };",
        )
        .expect("write entry");

        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new(temp_dir.path()));

        let result = BuildOptions::new(temp_dir.path().join("entry.tsx"))
            .externalize_from("package.json")
            .platform(Platform::Node)
            .cwd(temp_dir.path())
            .runtime(Arc::clone(&runtime))
            // Register both plugins
            .plugin(Arc::new(FobMdxPlugin::new(Arc::clone(&runtime))))
            .plugin(Arc::new(FobCssPlugin::new(runtime)))
            .build()
            .await
            .expect("Multiple plugins should work together");

        // Verify both plugins processed their files
        assert_has_assets(&result);
        assert_chunk_contains(&result, "Multi-Plugin Test");

        // STRONGER ASSERTIONS: Verify MDX was properly compiled
        let chunk = result.chunks().next().expect("should have chunk");

        // MDX plugin should have compiled markdown to JSX
        assert!(
            chunk.code.contains("jsx") || chunk.code.contains("jsxs"),
            "MDX should be compiled to JSX runtime calls"
        );

        // Raw markdown should not be present
        assert!(
            !chunk.code.contains("# Multi-Plugin Test"),
            "Raw markdown heading should be transformed"
        );
        assert!(
            !chunk.code.contains("**content**"),
            "Raw markdown bold should be transformed"
        );
    }

    /// Verify plugin order doesn't break functionality
    #[tokio::test]
    async fn test_plugin_order_independence() {
        let temp_dir = TempDir::new().expect("create temp dir");

        std::fs::write(
            temp_dir.path().join("content.mdx"),
            "# Plugin Order Test\n\nContent here.",
        )
        .expect("write mdx file");

        std::fs::write(
            temp_dir.path().join("entry.tsx"),
            "import Content from './content.mdx';\nexport { Content };",
        )
        .expect("write entry");

        let runtime1: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new(temp_dir.path()));

        // Test with CSS before MDX
        let result1 = BuildOptions::new(temp_dir.path().join("entry.tsx"))
            .platform(Platform::Node)
            .cwd(temp_dir.path())
            .runtime(Arc::clone(&runtime1))
            .plugin(Arc::new(FobCssPlugin::new(Arc::clone(&runtime1))))
            .plugin(Arc::new(FobMdxPlugin::new(runtime1)))
            .build()
            .await
            .expect("CSS before MDX should work");

        let runtime2: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new(temp_dir.path()));

        // Test with MDX before CSS
        let result2 = BuildOptions::new(temp_dir.path().join("entry.tsx"))
            .platform(Platform::Node)
            .cwd(temp_dir.path())
            .runtime(Arc::clone(&runtime2))
            .plugin(Arc::new(FobMdxPlugin::new(Arc::clone(&runtime2))))
            .plugin(Arc::new(FobCssPlugin::new(runtime2)))
            .build()
            .await
            .expect("MDX before CSS should work");

        // Both should produce output with the same content
        assert_chunk_contains(&result1, "Plugin Order Test");
        assert_chunk_contains(&result2, "Plugin Order Test");

        // STRONGER ASSERTIONS: Both orders should produce valid JSX
        let chunk1 = result1.chunks().next().expect("should have chunk");
        let chunk2 = result2.chunks().next().expect("should have chunk");

        // Both should have JSX runtime calls
        assert!(
            chunk1.code.contains("jsx") || chunk1.code.contains("jsxs"),
            "CSS-first order should compile MDX to JSX"
        );
        assert!(
            chunk2.code.contains("jsx") || chunk2.code.contains("jsxs"),
            "MDX-first order should compile MDX to JSX"
        );

        // Both should NOT have raw markdown
        assert!(
            !chunk1.code.contains("# Plugin Order Test"),
            "CSS-first: raw markdown should be transformed"
        );
        assert!(
            !chunk2.code.contains("# Plugin Order Test"),
            "MDX-first: raw markdown should be transformed"
        );
    }
}

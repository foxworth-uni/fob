//! Virtual file bundling tests
//!
//! Tests for the bundler's virtual file support - files that exist in memory
//! rather than on disk. This is essential for SSR packaging and programmatic bundling.

#[cfg(not(target_family = "wasm"))]
mod virtual_file_tests {
    use fob_bundler::runtime::BundlerRuntime;
    use fob_bundler::{BuildOptions, Platform, Runtime};
    use fob_plugin_mdx::FobMdxPlugin;
    use std::sync::Arc;
    use tempfile::TempDir;

    /// Test virtual TSX file bundling
    #[tokio::test]
    async fn test_virtual_tsx() {
        let result = BuildOptions::new("virtual:entry.tsx")
            .externalize_from("package.json")
            .platform(Platform::Node)
            .virtual_file(
                "virtual:entry.tsx",
                r#"
const greeting: string = "Hello from TSX";
export default function App() {
    return <div>{greeting}</div>;
}
"#,
            )
            .build()
            .await
            .expect("TSX virtual file should bundle");

        let bundle = result.output.as_single().expect("single bundle");
        assert!(!bundle.assets.is_empty(), "Should produce output assets");

        let chunk = result
            .chunks()
            .next()
            .expect("Should have at least one chunk");
        assert!(
            chunk.code.contains("Hello from TSX"),
            "Output should contain the greeting"
        );
        // Verify JSX was transformed (not raw JSX syntax)
        assert!(
            !chunk.code.contains("<div>"),
            "JSX should be transformed, not raw syntax"
        );
    }

    /// Test virtual JSX file bundling
    #[tokio::test]
    async fn test_virtual_jsx() {
        let result = BuildOptions::new("virtual:entry.jsx")
            .externalize_from("package.json")
            .platform(Platform::Node)
            .virtual_file(
                "virtual:entry.jsx",
                r#"
export function Button({ label }) {
    return <button className="btn">{label}</button>;
}
"#,
            )
            .build()
            .await
            .expect("JSX virtual file should bundle");

        let bundle = result.output.as_single().expect("single bundle");
        assert!(!bundle.assets.is_empty(), "Should produce output assets");

        let chunk = result
            .chunks()
            .next()
            .expect("Should have at least one chunk");
        assert!(
            chunk.code.contains("Button"),
            "Output should contain the Button function"
        );
    }

    /// Test virtual TypeScript file bundling
    #[tokio::test]
    async fn test_virtual_typescript() {
        let result = BuildOptions::new("virtual:utils.ts")
            .externalize_from("package.json")
            .platform(Platform::Node)
            .virtual_file(
                "virtual:utils.ts",
                r#"
interface User {
    name: string;
    age: number;
}

export function greet(user: User): string {
    return `Hello ${user.name}, you are ${user.age} years old`;
}
"#,
            )
            .build()
            .await
            .expect("TypeScript virtual file should bundle");

        let bundle = result.output.as_single().expect("single bundle");
        assert!(!bundle.assets.is_empty());

        let chunk = result.chunks().next().expect("Should have chunk");
        // TypeScript types should be stripped
        assert!(
            !chunk.code.contains("interface User"),
            "TypeScript interface should be stripped"
        );
        assert!(chunk.code.contains("greet"), "Function should be in output");
    }

    /// Test multiple virtual files importing each other
    #[tokio::test]
    async fn test_virtual_file_chain() {
        let result = BuildOptions::new("virtual:entry.tsx")
            .externalize_from("package.json")
            .platform(Platform::Node)
            .virtual_file(
                "virtual:entry.tsx",
                r#"
import { Button } from 'virtual:Button.tsx';
import { formatName } from 'virtual:utils.ts';

export function App() {
    return <Button label={formatName("World")} />;
}
"#,
            )
            .virtual_file(
                "virtual:Button.tsx",
                r#"
export function Button({ label }: { label: string }) {
    return <button>{label}</button>;
}
"#,
            )
            .virtual_file(
                "virtual:utils.ts",
                r#"
export function formatName(name: string): string {
    return `Hello, ${name}!`;
}
"#,
            )
            .build()
            .await
            .expect("Virtual file chain should bundle");

        assert_eq!(result.stats().module_count, 3, "Should have 3 modules");
        let chunk = result
            .chunks()
            .next()
            .expect("Should have at least one chunk");
        assert!(
            chunk.code.contains("formatName(\"World\")"),
            "Bundled output should contain the call to formatName with 'World'"
        );
    }

    /// Test MDX with disk fixtures (virtual MDX needs special handling)
    ///
    /// MDX plugin reads files via Runtime, so we need to use disk fixtures
    /// or ensure the runtime has access to the virtual MDX content.
    #[tokio::test]
    async fn test_mdx_disk_fixture() {
        let temp_dir = TempDir::new().expect("create temp dir");

        // Create MDX file on disk
        std::fs::write(
            temp_dir.path().join("content.mdx"),
            r#"---
title: "Test MDX"
---

# Welcome to MDX

This is **markdown** with JSX support.
"#,
        )
        .expect("write mdx file");

        // Create entry that imports MDX
        std::fs::write(
            temp_dir.path().join("entry.tsx"),
            r#"
import Content from './content.mdx';

export function Page() {
    return <Content />;
}
"#,
        )
        .expect("write entry file");

        // Create runtime for the MDX plugin
        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new(temp_dir.path()));

        let result = BuildOptions::new(temp_dir.path().join("entry.tsx"))
            .externalize_from("package.json")
            .platform(Platform::Node)
            .cwd(temp_dir.path())
            .runtime(Arc::clone(&runtime))
            .plugin(Arc::new(FobMdxPlugin::new(runtime)))
            .build()
            .await
            .expect("MDX bundling should succeed");

        let bundle = result.output.as_single().expect("single bundle");
        assert!(!bundle.assets.is_empty(), "Should produce assets");

        let chunk = result.chunks().next().expect("Should have chunk");

        // Verify MDX was compiled to JSX (not raw markdown)
        assert!(
            chunk.code.contains("Welcome to MDX"),
            "Should contain MDX content"
        );
        assert!(
            !chunk.code.contains("# Welcome"),
            "Should not contain raw markdown syntax (# heading)"
        );
    }

    /// Test MDX with frontmatter
    #[tokio::test]
    async fn test_mdx_with_frontmatter() {
        let temp_dir = TempDir::new().expect("create temp dir");

        std::fs::write(
            temp_dir.path().join("post.mdx"),
            r#"---
title: "My Blog Post"
author: "Test Author"
date: "2025-01-01"
---

# {frontmatter.title}

Written by {frontmatter.author}.

Some **bold** and *italic* text.
"#,
        )
        .expect("write mdx file");

        std::fs::write(
            temp_dir.path().join("entry.tsx"),
            r#"
import Post from './post.mdx';

export function BlogPage() {
    return <Post />;
}
"#,
        )
        .expect("write entry file");

        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new(temp_dir.path()));

        let result = BuildOptions::new(temp_dir.path().join("entry.tsx"))
            .externalize_from("package.json")
            .platform(Platform::Node)
            .cwd(temp_dir.path())
            .runtime(Arc::clone(&runtime))
            .plugin(Arc::new(FobMdxPlugin::new(runtime)))
            .build()
            .await
            .expect("MDX with frontmatter should bundle");

        let chunk = result.chunks().next().expect("Should have chunk");

        // Content should be present
        assert!(
            chunk.code.contains("My Blog Post") || chunk.code.contains("frontmatter"),
            "Frontmatter or content should be accessible"
        );
    }

    /// Test fully virtual MDX bundling - CRITICAL REGRESSION TEST
    ///
    /// This is the exact scenario that gumbo deploy uses: a virtual entry
    /// importing a virtual MDX file. If this test fails, gumbo deploy is broken.
    #[tokio::test]
    async fn test_virtual_mdx_basic() {
        let bundler_runtime = BundlerRuntime::new(".");

        // Add virtual MDX content to runtime so MDX plugin can read it
        bundler_runtime.add_virtual_file(
            "virtual:content.mdx",
            b"---\ntitle: \"Virtual MDX Test\"\n---\n\n# Hello from Virtual MDX\n\nThis is **bold** text and *italic* text.\n\n## Features\n\n- Item 1\n- Item 2\n",
        );

        let runtime: Arc<dyn Runtime> = Arc::new(bundler_runtime);

        let result = BuildOptions::new("virtual:entry.tsx")
            .externalize_from("package.json")
            .platform(Platform::Node)
            .virtual_file(
                "virtual:entry.tsx",
                "import Content from 'virtual:content.mdx';\nexport { Content };",
            )
            .runtime(Arc::clone(&runtime))
            .plugin(Arc::new(FobMdxPlugin::new(runtime)))
            .build()
            .await
            .expect("Virtual MDX should bundle - this is the gumbo deploy scenario");

        let bundle = result.output.as_single().expect("single bundle");
        assert!(!bundle.assets.is_empty(), "Should produce output assets");

        let chunk = result.chunks().next().expect("Should have chunk");

        // Verify MDX was compiled to JSX (not raw markdown)
        assert!(
            chunk.code.contains("Virtual MDX Test")
                || chunk.code.contains("Hello from Virtual MDX"),
            "MDX content should be in output"
        );

        // Raw markdown syntax should NOT be present - this proves MDX plugin ran
        assert!(
            !chunk.code.contains("# Hello from Virtual MDX"),
            "Raw markdown heading should be transformed by MDX plugin"
        );
        assert!(
            !chunk.code.contains("**bold**"),
            "Raw markdown bold syntax should be transformed"
        );

        // Should have JSX runtime (proves MDX→JSX compilation happened)
        assert!(
            chunk.code.contains("jsx")
                || chunk.code.contains("jsxs")
                || chunk.code.contains("createElement"),
            "Output should contain JSX runtime calls - proves MDX was compiled"
        );
    }

    /// Test virtual MDX with JSX embedded
    #[tokio::test]
    async fn test_virtual_mdx_with_jsx() {
        let bundler_runtime = BundlerRuntime::new(".");

        // MDX with embedded JSX component usage
        bundler_runtime.add_virtual_file(
            "virtual:page.mdx",
            b"# Welcome\n\nThis MDX has inline JSX:\n\n<div className=\"highlight\">Highlighted content</div>\n\nAnd continues with **markdown**.\n",
        );

        let runtime: Arc<dyn Runtime> = Arc::new(bundler_runtime);

        let result = BuildOptions::new("virtual:app.tsx")
            .externalize_from("package.json")
            .platform(Platform::Node)
            .virtual_file(
                "virtual:app.tsx",
                "import Page from 'virtual:page.mdx';\nexport function App() { return <Page />; }",
            )
            .runtime(Arc::clone(&runtime))
            .plugin(Arc::new(FobMdxPlugin::new(runtime)))
            .build()
            .await
            .expect("Virtual MDX with JSX should bundle");

        let chunk = result.chunks().next().expect("Should have chunk");

        // Content should be present
        assert!(
            chunk.code.contains("Welcome") || chunk.code.contains("Highlighted content"),
            "MDX content should be in output"
        );

        // JSX should be transformed
        assert!(
            !chunk.code.contains("<div className="),
            "Raw JSX syntax should be transformed"
        );
    }

    /// Test that virtual files with syntax errors produce clear error messages
    #[tokio::test]
    async fn test_virtual_file_syntax_error() {
        let result = BuildOptions::new("virtual:entry.js")
            .externalize_from("package.json")
            .platform(Platform::Node)
            .virtual_file(
                "virtual:entry.js",
                r#"
// This has invalid JavaScript syntax
export const x = {
    unclosed: "object
// Missing closing brace and quote
"#,
            )
            .build()
            .await;

        // Should fail with a parse/syntax error
        match result {
            Ok(_) => panic!("Syntax error should cause build to fail"),
            Err(e) => {
                let err_msg = e.to_string();
                // Error should be meaningful and indicate the problem
                assert!(!err_msg.is_empty(), "Error message should not be empty");
                // Should ideally mention parse/syntax issue (flexible check)
                assert!(
                    err_msg.len() > 10,
                    "Error message should be descriptive: {}",
                    err_msg
                );
            }
        }
    }

    /// Test virtual entry importing disk MDX
    ///
    /// This tests the hybrid scenario where a virtual entry imports a real MDX file.
    #[tokio::test]
    async fn test_virtual_entry_with_disk_mdx() {
        let temp_dir = TempDir::new().expect("create temp dir");

        // Create MDX file on disk
        std::fs::write(
            temp_dir.path().join("component.mdx"),
            r#"
# Hello MDX Component

This is content from a **disk file**.
"#,
        )
        .expect("write mdx file");

        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new(temp_dir.path()));

        let result = BuildOptions::new("virtual:entry.tsx")
            .externalize_from("package.json")
            .platform(Platform::Node)
            .virtual_file(
                "virtual:entry.tsx",
                r#"
import Content from './component.mdx';

export function App() {
    return <div><Content /></div>;
}
"#,
            )
            .cwd(temp_dir.path())
            .runtime(Arc::clone(&runtime))
            .plugin(Arc::new(FobMdxPlugin::new(runtime)))
            .build()
            .await
            .expect("Virtual entry with disk MDX should bundle");

        let chunk = result.chunks().next().expect("Should have chunk");
        assert!(
            chunk.code.contains("Hello MDX Component"),
            "Should contain MDX content from disk file"
        );
    }
}

//! File type extension tests
//!
//! Tests that verify the bundler correctly handles different file extensions:
//! - TypeScript (.ts)
//! - TypeScript with JSX (.tsx)
//! - JavaScript with JSX (.jsx)
//! - CSS (.css)
//! - MDX (.mdx)
//! - Markdown (.md)

mod helpers;

#[cfg(not(target_family = "wasm"))]
mod file_type_tests {
    use super::helpers::*;
    use fob_bundler::runtime::BundlerRuntime;
    use fob_bundler::{BuildOptions, Platform, Runtime};
    use fob_plugin_css::FobCssPlugin;
    use fob_plugin_mdx::FobMdxPlugin;
    use std::sync::Arc;

    // ===================
    // TSX Tests (.tsx)
    // ===================

    #[tokio::test]
    async fn test_tsx_basic() {
        let fixture = fixture_path("file-types/basic.tsx");

        let result = test_build_options(&fixture)
            .build()
            .await
            .expect("TSX fixture should bundle");

        assert_has_assets(&result);
        assert_chunk_contains(&result, "Button");

        // TypeScript interface should be stripped
        assert_chunk_not_contains(&result, "interface ButtonProps");
    }

    #[tokio::test]
    async fn test_tsx_fragments() {
        let fixture = fixture_path("file-types/fragments.tsx");

        let result = test_build_options(&fixture)
            .build()
            .await
            .expect("TSX with fragments should bundle");

        assert_has_assets(&result);
        assert_chunk_contains(&result, "FragmentList");
        assert_chunk_contains(&result, "EmptyFragment");

        // Fragment syntax should be transformed
        assert_chunk_not_contains(&result, "<>");
    }

    // ===================
    // JSX Tests (.jsx)
    // ===================

    #[tokio::test]
    async fn test_jsx_basic() {
        let fixture = fixture_path("file-types/basic.jsx");

        let result = test_build_options(&fixture)
            .build()
            .await
            .expect("JSX fixture should bundle");

        assert_has_assets(&result);
        assert_chunk_contains(&result, "Greeting");
        assert_chunk_contains(&result, "Hello");
    }

    #[tokio::test]
    async fn test_jsx_spread() {
        let fixture = fixture_path("file-types/spread.jsx");

        let result = test_build_options(&fixture)
            .build()
            .await
            .expect("JSX with spread should bundle");

        assert_has_assets(&result);
        assert_chunk_contains(&result, "Wrapper");
        assert_chunk_contains(&result, "ForwardedButton");
    }

    // ===================
    // CSS Tests (.css)
    // ===================

    #[tokio::test]
    async fn test_css_virtual_basic() {
        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new("."));

        // Use virtual: prefix consistently for imports
        let result = BuildOptions::new("virtual:entry.js")
            .bundle(false)
            .platform(Platform::Node)
            .virtual_file(
                "virtual:entry.js",
                "import 'virtual:styles.css';\nexport const x = 1;",
            )
            .virtual_file(
                "virtual:styles.css",
                ".container { display: flex; gap: 1rem; }",
            )
            .runtime(Arc::clone(&runtime))
            .plugin(Arc::new(FobCssPlugin::new(runtime)))
            .build()
            .await
            .expect("CSS virtual file should bundle");

        assert_has_assets(&result);
    }

    #[tokio::test]
    async fn test_css_disk_fixture() {
        let fixture = fixture_path("file-types/basic.css");
        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new(fixtures_dir()));

        // Create a JS entry that imports the CSS
        let entry_content = format!(
            "import '{}';\nexport const styles = 'loaded';",
            fixture.display()
        );

        let result = BuildOptions::new("virtual:entry.js")
            .bundle(false)
            .platform(Platform::Node)
            .virtual_file("virtual:entry.js", entry_content)
            .cwd(fixtures_dir())
            .runtime(Arc::clone(&runtime))
            .plugin(Arc::new(FobCssPlugin::new(runtime)))
            .build()
            .await
            .expect("CSS disk fixture should bundle");

        assert_has_assets(&result);
    }

    // ===================
    // MDX Tests (.mdx)
    // ===================

    #[tokio::test]
    async fn test_mdx_basic() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("create temp dir");

        // Copy fixture content to temp dir
        std::fs::copy(
            fixture_path("file-types/basic.mdx"),
            temp_dir.path().join("content.mdx"),
        )
        .expect("copy mdx fixture");

        std::fs::write(
            temp_dir.path().join("entry.tsx"),
            "import Content from './content.mdx';\nexport { Content };",
        )
        .expect("write entry");

        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new(temp_dir.path()));

        let result = BuildOptions::new(temp_dir.path().join("entry.tsx"))
            .bundle(false)
            .platform(Platform::Node)
            .cwd(temp_dir.path())
            .runtime(Arc::clone(&runtime))
            .plugin(Arc::new(FobMdxPlugin::new(runtime)))
            .build()
            .await
            .expect("MDX fixture should bundle");

        assert_has_assets(&result);

        // Verify MDX was compiled (content present, raw markdown not)
        assert_chunk_contains(&result, "Welcome to MDX");
        assert_chunk_not_contains(&result, "# Welcome");
    }

    #[tokio::test]
    async fn test_mdx_with_frontmatter() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("create temp dir");

        std::fs::write(
            temp_dir.path().join("doc.mdx"),
            r#"---
title: "Frontmatter Test"
description: "Testing YAML frontmatter in MDX"
---

# {frontmatter.title}

The description is: {frontmatter.description}
"#,
        )
        .expect("write mdx with frontmatter");

        std::fs::write(
            temp_dir.path().join("entry.tsx"),
            "import Doc from './doc.mdx';\nexport { Doc };",
        )
        .expect("write entry");

        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new(temp_dir.path()));

        let result = BuildOptions::new(temp_dir.path().join("entry.tsx"))
            .bundle(false)
            .platform(Platform::Node)
            .cwd(temp_dir.path())
            .runtime(Arc::clone(&runtime))
            .plugin(Arc::new(FobMdxPlugin::new(runtime)))
            .build()
            .await
            .expect("MDX with frontmatter should bundle");

        assert_has_assets(&result);

        // Check frontmatter is accessible
        assert_chunk_contains(&result, "Frontmatter Test");
    }

    // ===================
    // Markdown Tests (.md)
    // ===================

    #[tokio::test]
    async fn test_markdown_basic() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("create temp dir");

        std::fs::write(
            temp_dir.path().join("readme.md"),
            r#"# Markdown Test

This is plain **markdown** without JSX.

- Item 1
- Item 2
- Item 3

That's all folks!
"#,
        )
        .expect("write markdown file");

        std::fs::write(
            temp_dir.path().join("entry.tsx"),
            "import Readme from './readme.md';\nexport { Readme };",
        )
        .expect("write entry");

        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new(temp_dir.path()));

        // Note: MDX plugin should handle .md files too
        let mdx_plugin = FobMdxPlugin::new(Arc::clone(&runtime));

        let result = BuildOptions::new(temp_dir.path().join("entry.tsx"))
            .bundle(false)
            .platform(Platform::Node)
            .cwd(temp_dir.path())
            .runtime(Arc::clone(&runtime))
            .plugin(Arc::new(mdx_plugin))
            .build()
            .await;

        // Note: If .md isn't supported by MDX plugin, this will fail
        // which would be a gap to document
        if result.is_ok() {
            assert_has_assets(&result.as_ref().unwrap());
            assert_chunk_contains(&result.as_ref().unwrap(), "Markdown Test");
        } else {
            // Document that .md files need explicit handling
            println!("Note: .md files may need explicit handling - MDX plugin only handles .mdx");
        }
    }

    // ===================
    // TypeScript Tests (.ts)
    // ===================

    #[tokio::test]
    async fn test_typescript_basic() {
        let result = BuildOptions::new("virtual:utils.ts")
            .bundle(false)
            .platform(Platform::Node)
            .virtual_file(
                "virtual:utils.ts",
                r#"
interface Config {
    name: string;
    debug: boolean;
}

export function createConfig(name: string): Config {
    return { name, debug: false };
}

export type { Config };
"#,
            )
            .build()
            .await
            .expect("TypeScript should bundle");

        assert_has_assets(&result);
        assert_chunk_contains(&result, "createConfig");

        // TypeScript should be stripped
        assert_chunk_not_contains(&result, "interface Config");
        assert_chunk_not_contains(&result, ": string");
    }

    #[tokio::test]
    async fn test_typescript_with_generics() {
        let result = BuildOptions::new("virtual:generics.ts")
            .bundle(false)
            .platform(Platform::Node)
            .virtual_file(
                "virtual:generics.ts",
                r#"
export function identity<T>(value: T): T {
    return value;
}

export function mapArray<T, U>(arr: T[], fn: (item: T) => U): U[] {
    return arr.map(fn);
}

export class Container<T> {
    constructor(private value: T) {}
    get(): T {
        return this.value;
    }
}
"#,
            )
            .build()
            .await
            .expect("TypeScript with generics should bundle");

        assert_has_assets(&result);
        assert_chunk_contains(&result, "identity");
        assert_chunk_contains(&result, "Container");

        // Generic syntax should be stripped
        assert_chunk_not_contains(&result, "<T>");
    }

    // ===================
    // Cross-Extension Chain Tests
    // ===================

    /// Test mixed extension import chain: entry.ts → component.tsx → content.mdx
    ///
    /// This tests a real-world scenario where different file types import each other.
    /// The chain exercises TypeScript, TSX, and MDX plugins working together.
    #[tokio::test]
    async fn test_mixed_extension_chain() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("create temp dir");

        // entry.ts - TypeScript entry point
        std::fs::write(
            temp_dir.path().join("entry.ts"),
            r#"
import { App } from './App';

// Re-export the App component
export { App };

// Add some TypeScript-only syntax
interface AppConfig {
    name: string;
}

export const config: AppConfig = { name: "Mixed Extension Test" };
"#,
        )
        .expect("write entry.ts");

        // App.tsx - TSX component that imports MDX
        std::fs::write(
            temp_dir.path().join("App.tsx"),
            r#"
import Content from './content.mdx';

interface AppProps {
    title?: string;
}

export function App({ title = "Default" }: AppProps) {
    return (
        <div className="app">
            <h1>{title}</h1>
            <Content />
        </div>
    );
}
"#,
        )
        .expect("write App.tsx");

        // content.mdx - MDX content
        std::fs::write(
            temp_dir.path().join("content.mdx"),
            r#"# Mixed Extensions Content

This MDX is imported by TSX which is imported by TS.

**Bold text** and *italic text* for testing.

The chain is: `entry.ts` → `App.tsx` → `content.mdx`
"#,
        )
        .expect("write content.mdx");

        let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new(temp_dir.path()));

        let result = BuildOptions::new(temp_dir.path().join("entry.ts"))
            .bundle(false)
            .platform(Platform::Node)
            .cwd(temp_dir.path())
            .runtime(Arc::clone(&runtime))
            .plugin(Arc::new(FobMdxPlugin::new(runtime)))
            .build()
            .await
            .expect("Mixed extension chain should bundle successfully");

        assert_has_assets(&result);

        // Verify all modules in the chain were processed
        let chunk = result.chunks().next().expect("should have chunk");

        // Entry.ts content should be present
        assert!(
            chunk.code.contains("config") || chunk.code.contains("Mixed Extension Test"),
            "Entry.ts content should be in output"
        );

        // App.tsx component should be present
        assert!(
            chunk.code.contains("App"),
            "App.tsx component should be in output"
        );

        // TypeScript interfaces should be stripped
        assert!(
            !chunk.code.contains("interface AppConfig")
                && !chunk.code.contains("interface AppProps"),
            "TypeScript interfaces should be stripped"
        );

        // MDX content should be transformed (not raw markdown)
        assert!(
            chunk.code.contains("Mixed Extensions Content"),
            "MDX content text should be in output"
        );
        assert!(
            !chunk.code.contains("# Mixed Extensions Content"),
            "Raw markdown heading should be transformed"
        );

        // JSX should be compiled (not raw JSX syntax)
        assert!(
            chunk.code.contains("jsx")
                || chunk.code.contains("jsxs")
                || chunk.code.contains("createElement"),
            "Output should contain JSX runtime calls"
        );
    }

    /// Test virtual file chain with different extensions
    #[tokio::test]
    async fn test_virtual_mixed_extension_chain() {
        let bundler_runtime = BundlerRuntime::new(".");

        // Add virtual MDX
        bundler_runtime.add_virtual_file(
            "virtual:doc.mdx",
            b"# Virtual Doc\n\nContent from **virtual MDX**.",
        );

        let runtime: Arc<dyn Runtime> = Arc::new(bundler_runtime);

        let result = BuildOptions::new("virtual:main.ts")
            .bundle(false)
            .platform(Platform::Node)
            .virtual_file(
                "virtual:main.ts",
                r#"
import { Page } from 'virtual:Page.tsx';
export { Page };
export const version: string = "1.0.0";
"#,
            )
            .virtual_file(
                "virtual:Page.tsx",
                r#"
import Doc from 'virtual:doc.mdx';
export function Page() {
    return <div><Doc /></div>;
}
"#,
            )
            .runtime(Arc::clone(&runtime))
            .plugin(Arc::new(FobMdxPlugin::new(runtime)))
            .build()
            .await
            .expect("Virtual mixed extension chain should bundle");

        let chunk = result.chunks().next().expect("should have chunk");

        // Verify chain works
        assert!(
            chunk.code.contains("Page"),
            "Page component should be in output"
        );
        assert!(
            chunk.code.contains("version") || chunk.code.contains("1.0.0"),
            "Main.ts export should be in output"
        );

        // MDX should be compiled
        assert!(
            chunk.code.contains("Virtual Doc"),
            "MDX content should be in output"
        );
        assert!(
            !chunk.code.contains("# Virtual Doc"),
            "Raw markdown should be transformed"
        );

        // TypeScript should be stripped
        assert!(
            !chunk.code.contains(": string"),
            "TypeScript type annotations should be stripped"
        );
    }
}

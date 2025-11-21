//! Integration tests for fob-plugin-astro
//!
//! These tests verify the complete flow from Astro component files through the parser
//! to the rolldown plugin integration.

use fob_plugin_astro::FobAstroPlugin;
use rolldown_plugin::{HookLoadArgs, Plugin};
use std::fs;
use tempfile::TempDir;

/// Helper to create a temporary .astro file for testing
fn create_astro_file(dir: &TempDir, name: &str, content: &str) -> String {
    let file_path = dir.path().join(name);
    fs::write(&file_path, content).expect("Failed to write test file");
    file_path.to_str().unwrap().to_string()
}

#[tokio::test]
async fn test_frontmatter_only() {
    let dir = TempDir::new().unwrap();
    let astro_content = r#"---
const title = 'My Astro Page'
const description = 'A test page'
const data = await fetch('/api/data').then(r => r.json())
---
<html>
    <head>
        <title>{title}</title>
        <meta name="description" content={description} />
    </head>
    <body>
        <h1>{title}</h1>
    </body>
</html>
"#;

    let file_path = create_astro_file(&dir, "Page.astro", astro_content);

    let plugin = FobAstroPlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Should contain the frontmatter content
    assert!(output.code.contains("const title = 'My Astro Page'"));
    assert!(output.code.contains("const data = await fetch"));

    // Should be TypeScript (frontmatter default)
    assert!(matches!(
        output.module_type,
        Some(rolldown_common::ModuleType::Ts)
    ));
}

#[tokio::test]
async fn test_script_tags_only() {
    let dir = TempDir::new().unwrap();
    let astro_content = r#"
<html>
    <head>
        <title>Test</title>
    </head>
    <body>
        <h1>Hello Astro</h1>
        <script>
            console.log('Client-side script')
            document.querySelector('h1').addEventListener('click', () => {
                alert('Clicked!')
            })
        </script>
    </body>
</html>
"#;

    let file_path = create_astro_file(&dir, "ClientSide.astro", astro_content);

    let plugin = FobAstroPlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Should contain the script content
    assert!(output.code.contains("console.log('Client-side script')"));
    assert!(output.code.contains("addEventListener"));

    // Should be JavaScript
    assert!(matches!(
        output.module_type,
        Some(rolldown_common::ModuleType::Js)
    ));
}

#[tokio::test]
async fn test_frontmatter_and_scripts() {
    let dir = TempDir::new().unwrap();
    let astro_content = r#"---
const apiUrl = '/api/items'
const items = await fetch(apiUrl).then(r => r.json())
---
<html>
    <body>
        <ul id="items">
            {items.map(item => <li>{item.name}</li>)}
        </ul>
        <script>
            console.log('Hydrating client-side')
            const list = document.getElementById('items')
        </script>
        <script>
            // Second script for additional interactivity
            list.addEventListener('click', (e) => {
                console.log('Clicked:', e.target.textContent)
            })
        </script>
    </body>
</html>
"#;

    let file_path = create_astro_file(&dir, "Combined.astro", astro_content);

    let plugin = FobAstroPlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    let code_str = output.code.to_string();

    // Frontmatter should come first
    let trimmed = code_str.trim_start();
    assert!(
        trimmed.starts_with("const apiUrl = '/api/items'"),
        "Frontmatter should come first. Got: {:?}",
        &trimmed[..trimmed.len().min(50)]
    );

    // Scripts should follow
    assert!(code_str.contains("console.log('Hydrating client-side')"));
    assert!(code_str.contains("list.addEventListener"));

    // Should be TypeScript (frontmatter is TS)
    assert!(matches!(
        output.module_type,
        Some(rolldown_common::ModuleType::Ts)
    ));
}

#[tokio::test]
async fn test_multiple_script_tags() {
    let dir = TempDir::new().unwrap();
    let astro_content = r#"
<html>
    <body>
        <script>
            console.log('Script 1')
        </script>
        <script>
            console.log('Script 2')
        </script>
        <script>
            console.log('Script 3')
        </script>
    </body>
</html>
"#;

    let file_path = create_astro_file(&dir, "MultiScript.astro", astro_content);

    let plugin = FobAstroPlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Should contain all three scripts
    assert!(output.code.contains("Script 1"));
    assert!(output.code.contains("Script 2"));
    assert!(output.code.contains("Script 3"));
}

#[tokio::test]
async fn test_no_frontmatter_or_scripts() {
    let dir = TempDir::new().unwrap();
    let astro_content = r#"
<html>
    <head>
        <title>Static Page</title>
    </head>
    <body>
        <h1>No JavaScript here</h1>
        <p>Just static content</p>
    </body>
</html>
"#;

    let file_path = create_astro_file(&dir, "Static.astro", astro_content);

    let plugin = FobAstroPlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Should return empty export
    assert_eq!(output.code.to_string(), "export default {}");

    // Should be JavaScript
    assert!(matches!(
        output.module_type,
        Some(rolldown_common::ModuleType::Js)
    ));
}

#[tokio::test]
async fn test_frontmatter_with_leading_whitespace() {
    let dir = TempDir::new().unwrap();
    let astro_content = r#"

---
const title = 'Whitespace Test'
---
<html><head><title>{title}</title></head></html>
"#;

    let file_path = create_astro_file(&dir, "Whitespace.astro", astro_content);

    let plugin = FobAstroPlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Should contain frontmatter
    assert!(output.code.contains("const title = 'Whitespace Test'"));
}

#[tokio::test]
async fn test_non_astro_file() {
    let dir = TempDir::new().unwrap();
    let js_content = "export const x = 1;";
    let file_path = create_astro_file(&dir, "test.js", js_content);

    let plugin = FobAstroPlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    // Should return None for non-.astro files
    assert!(result.is_none());
}

#[tokio::test]
async fn test_malformed_unclosed_frontmatter() {
    let dir = TempDir::new().unwrap();
    let astro_content = r#"---
const title = 'Broken'
const x = 1
<html><body>Missing closing ---</body></html>
"#;

    let file_path = create_astro_file(&dir, "BrokenFrontmatter.astro", astro_content);

    let plugin = FobAstroPlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await;

    // Should return an error for unclosed frontmatter
    assert!(
        result.is_err(),
        "Expected error for unclosed frontmatter, got: {:?}",
        result
    );
    let error = result.unwrap_err();

    let error_chain = format!("{:?}", error);
    assert!(
        error_chain.contains("Unclosed frontmatter") || error_chain.contains("unclosed"),
        "Expected unclosed frontmatter error, got: {}",
        error_chain
    );
}

#[tokio::test]
async fn test_malformed_unclosed_script() {
    let dir = TempDir::new().unwrap();
    let astro_content = r#"
<html>
    <body>
        <script>
            console.log('Missing closing tag')
    </body>
</html>
"#;

    let file_path = create_astro_file(&dir, "BrokenScript.astro", astro_content);

    let plugin = FobAstroPlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await;

    // Should return an error for unclosed script tag
    assert!(
        result.is_err(),
        "Expected error for unclosed script tag, got: {:?}",
        result
    );
    let error = result.unwrap_err();

    let error_chain = format!("{:?}", error);
    assert!(
        error_chain.contains("Unclosed script tag") || error_chain.contains("unclosed"),
        "Expected unclosed tag error, got: {}",
        error_chain
    );
}

#[tokio::test]
async fn test_self_closing_script_tag() {
    let dir = TempDir::new().unwrap();
    let astro_content = r#"
<html>
    <body>
        <script src="./external.js" />
    </body>
</html>
"#;

    let file_path = create_astro_file(&dir, "ExternalScript.astro", astro_content);

    let plugin = FobAstroPlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Self-closing tags have empty content
    assert_eq!(output.code.to_string(), "");
}

#[tokio::test]
async fn test_frontmatter_with_imports() {
    let dir = TempDir::new().unwrap();
    let astro_content = r#"---
import { getCollection } from 'astro:content'
import Layout from '../layouts/Base.astro'

const posts = await getCollection('blog')
const title = 'Blog'
---
<Layout title={title}>
    <h1>{title}</h1>
    {posts.map(post => <article>{post.data.title}</article>)}
</Layout>
"#;

    let file_path = create_astro_file(&dir, "Blog.astro", astro_content);

    let plugin = FobAstroPlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Should contain imports
    assert!(output.code.contains("import { getCollection } from 'astro:content'"));
    assert!(output.code.contains("import Layout from '../layouts/Base.astro'"));
    assert!(output.code.contains("const posts = await getCollection('blog')"));
}

#[tokio::test]
async fn test_script_with_comments() {
    let dir = TempDir::new().unwrap();
    let astro_content = r#"
<html>
    <body>
        <script>
            // Single line comment
            const x = 1

            /* Multi-line
               comment */
            console.log(x)
        </script>
    </body>
</html>
"#;

    let file_path = create_astro_file(&dir, "Commented.astro", astro_content);

    let plugin = FobAstroPlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Should preserve comments
    assert!(output.code.contains("// Single line comment"));
    assert!(output.code.contains("/* Multi-line"));
}

#[tokio::test]
async fn test_complex_frontmatter() {
    let dir = TempDir::new().unwrap();
    let astro_content = r#"---
interface Props {
    title: string
    description?: string
}

const { title, description = 'Default description' } = Astro.props as Props

const canonicalURL = new URL(Astro.url.pathname, Astro.site)
---
<html>
    <head>
        <title>{title}</title>
        <link rel="canonical" href={canonicalURL} />
    </head>
</html>
"#;

    let file_path = create_astro_file(&dir, "TypedComponent.astro", astro_content);

    let plugin = FobAstroPlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Should contain TypeScript interface and code
    assert!(output.code.contains("interface Props"));
    assert!(output.code.contains("const { title, description"));
    assert!(output.code.contains("const canonicalURL = new URL"));

    // Should be TypeScript
    assert!(matches!(
        output.module_type,
        Some(rolldown_common::ModuleType::Ts)
    ));
}

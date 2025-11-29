//! Integration tests for fob-plugin-svelte
//!
//! These tests verify the complete flow from Svelte component files through the parser
//! to the rolldown plugin integration.

use fob_bundler::{HookLoadArgs, Plugin};
use fob_plugin_svelte::FobSveltePlugin;
use std::fs;
use tempfile::TempDir;

/// Helper to create a temporary .svelte file for testing
fn create_svelte_file(dir: &TempDir, name: &str, content: &str) -> String {
    let file_path = dir.path().join(name);
    fs::write(&file_path, content).expect("Failed to write test file");
    file_path.to_str().unwrap().to_string()
}

#[tokio::test]
async fn test_basic_script_extraction() {
    let dir = TempDir::new().unwrap();
    let svelte_content = r#"
<script>
let count = 0

function increment() {
    count += 1
}
</script>

<button on:click={increment}>
    Count: {count}
</button>

<style>
button {
    background: blue;
}
</style>
"#;

    let file_path = create_svelte_file(&dir, "Counter.svelte", svelte_content);

    let plugin = FobSveltePlugin::new();
    let ctx = fob_bundler::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Should contain the script content
    assert!(output.code.contains("let count = 0"));
    assert!(output.code.contains("function increment"));

    // Should be JavaScript
    assert!(matches!(
        output.module_type,
        Some(fob_bundler::ModuleType::Js)
    ));
}

#[tokio::test]
async fn test_module_context() {
    let dir = TempDir::new().unwrap();
    let svelte_content = r#"
<script context="module">
export const preload = async () => {
    return { data: [] }
}
</script>

<script>
import { onMount } from 'svelte'

let items = []

onMount(async () => {
    const response = await fetch('/api/items')
    items = await response.json()
})
</script>

<div>
    {#each items as item}
        <p>{item.name}</p>
    {/each}
</div>
"#;

    let file_path = create_svelte_file(&dir, "DataLoader.svelte", svelte_content);

    let plugin = FobSveltePlugin::new();
    let ctx = fob_bundler::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    let code_str = output.code.to_string();

    // Module context should come first
    let trimmed = code_str.trim_start();
    assert!(
        trimmed.starts_with("export const preload"),
        "Module context script should come first. Got: {:?}",
        &trimmed[..trimmed.len().min(50)]
    );

    // Instance script should come after
    assert!(code_str.contains("import { onMount } from 'svelte'"));
    assert!(code_str.contains("let items = []"));
}

#[tokio::test]
async fn test_typescript_support() {
    let dir = TempDir::new().unwrap();
    let svelte_content = r#"
<script lang="ts">
interface User {
    name: string
    age: number
}

let user: User = {
    name: 'Alice',
    age: 30
}
</script>

<div>{user.name}</div>
"#;

    let file_path = create_svelte_file(&dir, "TypedComponent.svelte", svelte_content);

    let plugin = FobSveltePlugin::new();
    let ctx = fob_bundler::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Should be TypeScript
    assert!(matches!(
        output.module_type,
        Some(fob_bundler::ModuleType::Ts)
    ));

    // Should contain TypeScript-specific code
    assert!(output.code.contains("interface User"));
    assert!(output.code.contains("let user: User"));
}

#[tokio::test]
async fn test_module_context_with_typescript() {
    let dir = TempDir::new().unwrap();
    let svelte_content = r#"
<script context="module" lang="ts">
export const shared: Map<string, number> = new Map()
</script>

<script lang="ts">
import { onMount } from 'svelte'

let value: number = 0

onMount(() => {
    shared.set('key', 42)
    value = shared.get('key') || 0
})
</script>

<div>{value}</div>
"#;

    let file_path = create_svelte_file(&dir, "ModuleTS.svelte", svelte_content);

    let plugin = FobSveltePlugin::new();
    let ctx = fob_bundler::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    let code_str = output.code.to_string();

    // Module context should come first
    let trimmed = code_str.trim_start();
    assert!(
        trimmed.starts_with("export const shared: Map<string, number>"),
        "Module context should come first"
    );

    // Instance script should follow
    assert!(code_str.contains("import { onMount } from 'svelte'"));
    assert!(code_str.contains("let value: number = 0"));

    // Should be TypeScript
    assert!(matches!(
        output.module_type,
        Some(fob_bundler::ModuleType::Ts)
    ));
}

#[tokio::test]
async fn test_no_script_blocks() {
    let dir = TempDir::new().unwrap();
    let svelte_content = r#"
<div>
    <h1>Hello World</h1>
    <p>This component has no script blocks</p>
</div>

<style>
h1 { color: blue; }
</style>
"#;

    let file_path = create_svelte_file(&dir, "NoScript.svelte", svelte_content);

    let plugin = FobSveltePlugin::new();
    let ctx = fob_bundler::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Should return empty export
    assert_eq!(output.code.to_string(), "export default {}");

    // Should be JavaScript
    assert!(matches!(
        output.module_type,
        Some(fob_bundler::ModuleType::Js)
    ));
}

#[tokio::test]
async fn test_non_svelte_file() {
    let dir = TempDir::new().unwrap();
    let js_content = "export const x = 1;";
    let file_path = create_svelte_file(&dir, "test.js", js_content);

    let plugin = FobSveltePlugin::new();
    let ctx = fob_bundler::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    // Should return None for non-.svelte files
    assert!(result.is_none());
}

#[tokio::test]
async fn test_malformed_svelte_unclosed_tag() {
    let dir = TempDir::new().unwrap();
    let svelte_content = r#"
<script>
let count = 0
<!-- Missing closing tag -->

<div>{count}</div>
"#;

    let file_path = create_svelte_file(&dir, "Broken.svelte", svelte_content);

    let plugin = FobSveltePlugin::new();
    let ctx = fob_bundler::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await;

    // Should return an error for unclosed script tag
    assert!(
        result.is_err(),
        "Expected error for unclosed script tag, got: {:?}",
        result
    );
    let error = result.unwrap_err();

    // Check the full error chain
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
    let svelte_content = r#"
<script src="./external.js" />

<div>External script</div>
"#;

    let file_path = create_svelte_file(&dir, "ExternalScript.svelte", svelte_content);

    let plugin = FobSveltePlugin::new();
    let ctx = fob_bundler::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Self-closing tags have empty content
    assert_eq!(output.code.to_string(), "");
}

#[tokio::test]
async fn test_script_with_comments() {
    let dir = TempDir::new().unwrap();
    let svelte_content = r#"
<script>
// Single line comment
let count = 0

/* Multi-line
   comment */
function increment() {
    count++
}
</script>

<button on:click={increment}>{count}</button>
"#;

    let file_path = create_svelte_file(&dir, "Commented.svelte", svelte_content);

    let plugin = FobSveltePlugin::new();
    let ctx = fob_bundler::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Should preserve comments
    assert!(output.code.contains("// Single line comment"));
    assert!(output.code.contains("/* Multi-line"));
    assert!(output.code.contains("let count = 0"));
}

#[tokio::test]
async fn test_reactive_declarations() {
    let dir = TempDir::new().unwrap();
    let svelte_content = r#"
<script>
let count = 0

$: doubled = count * 2
$: {
    console.log(`Count is ${count}`)
}
</script>

<div>{doubled}</div>
"#;

    let file_path = create_svelte_file(&dir, "Reactive.svelte", svelte_content);

    let plugin = FobSveltePlugin::new();
    let ctx = fob_bundler::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Should preserve reactive declarations
    assert!(output.code.contains("$: doubled = count * 2"));
    assert!(output.code.contains("$: {"));
}

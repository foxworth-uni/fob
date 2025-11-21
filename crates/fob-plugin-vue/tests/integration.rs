//! Integration tests for fob-plugin-vue
//!
//! These tests verify the complete flow from Vue SFC files through the parser
//! to the rolldown plugin integration.

use fob_plugin_vue::FobVuePlugin;
use rolldown_plugin::{HookLoadArgs, Plugin};
use std::fs;
use tempfile::TempDir;

/// Helper to create a temporary .vue file for testing
fn create_vue_file(dir: &TempDir, name: &str, content: &str) -> String {
    let file_path = dir.path().join(name);
    fs::write(&file_path, content).expect("Failed to write test file");
    file_path.to_str().unwrap().to_string()
}

#[tokio::test]
async fn test_basic_script_extraction() {
    let dir = TempDir::new().unwrap();
    let vue_content = r#"
<template>
  <div>Hello World</div>
</template>

<script>
export default {
  name: 'HelloWorld',
  data() {
    return {
      message: 'Hello'
    }
  }
}
</script>

<style scoped>
div {
  color: blue;
}
</style>
"#;

    let file_path = create_vue_file(&dir, "HelloWorld.vue", vue_content);

    let plugin = FobVuePlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Should contain the script content
    assert!(output.code.contains("export default"));
    assert!(output.code.contains("HelloWorld"));
    assert!(output.code.contains("message"));

    // Should be JavaScript
    assert!(matches!(
        output.module_type,
        Some(rolldown_common::ModuleType::Js)
    ));
}

#[tokio::test]
async fn test_script_setup() {
    let dir = TempDir::new().unwrap();
    let vue_content = r#"
<script setup>
import { ref } from 'vue'

const count = ref(0)
const increment = () => count.value++
</script>

<template>
  <button @click="increment">Count: {{ count }}</button>
</template>
"#;

    let file_path = create_vue_file(&dir, "Counter.vue", vue_content);

    let plugin = FobVuePlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Should contain script setup content
    assert!(output.code.contains("import { ref } from 'vue'"));
    assert!(output.code.contains("const count = ref(0)"));
    assert!(output.code.contains("increment"));
}

#[tokio::test]
async fn test_typescript_support() {
    let dir = TempDir::new().unwrap();
    let vue_content = r#"
<script lang="ts">
export default defineComponent({
  name: 'TypedComponent',
  props: {
    count: {
      type: Number as PropType<number>,
      required: true
    }
  }
})
</script>

<template>
  <div>{{ count }}</div>
</template>
"#;

    let file_path = create_vue_file(&dir, "TypedComponent.vue", vue_content);

    let plugin = FobVuePlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Should be TypeScript
    assert!(matches!(
        output.module_type,
        Some(rolldown_common::ModuleType::Ts)
    ));

    // Should contain TypeScript-specific code
    assert!(output.code.contains("PropType"));
    assert!(output.code.contains("defineComponent"));
}

#[tokio::test]
async fn test_multiple_scripts() {
    let dir = TempDir::new().unwrap();
    let vue_content = r#"
<script>
export default {
  name: 'MyComponent'
}
</script>

<script setup lang="ts">
import { ref } from 'vue'
const count = ref<number>(0)
</script>

<template>
  <div>{{ count }}</div>
</template>
"#;

    let file_path = create_vue_file(&dir, "MultiScript.vue", vue_content);

    let plugin = FobVuePlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Setup script should come first (after any leading whitespace)
    let code_str = output.code.to_string();
    let trimmed = code_str.trim_start();
    assert!(trimmed.starts_with("import { ref } from 'vue'"),
            "Setup script should come first. Got: {:?}", &trimmed[..trimmed.len().min(50)]);

    // Regular script should come after
    assert!(code_str.contains("export default"));
    assert!(code_str.contains("MyComponent"));

    // Should upgrade to TypeScript (from setup script)
    assert!(matches!(
        output.module_type,
        Some(rolldown_common::ModuleType::Ts)
    ));
}

#[tokio::test]
async fn test_no_script_blocks() {
    let dir = TempDir::new().unwrap();
    let vue_content = r#"
<template>
  <div>No script here</div>
</template>

<style>
div { color: red; }
</style>
"#;

    let file_path = create_vue_file(&dir, "NoScript.vue", vue_content);

    let plugin = FobVuePlugin::new();
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
async fn test_non_vue_file() {
    let dir = TempDir::new().unwrap();
    let js_content = "export const x = 1;";
    let file_path = create_vue_file(&dir, "test.js", js_content);

    let plugin = FobVuePlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    // Should return None for non-.vue files
    assert!(result.is_none());
}

#[tokio::test]
async fn test_malformed_vue_unclosed_tag() {
    let dir = TempDir::new().unwrap();
    let vue_content = r#"
<template>
  <div>Test</div>
</template>

<script>
export default {
  name: 'Broken'
}
<!-- Missing closing tag -->
"#;

    let file_path = create_vue_file(&dir, "Broken.vue", vue_content);

    let plugin = FobVuePlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await;

    // Should return an error for unclosed script tag
    assert!(result.is_err(), "Expected error for unclosed script tag, got: {:?}", result);
    let error = result.unwrap_err();

    // Check the full error chain (error is wrapped by anyhow::Context)
    let error_chain = format!("{:?}", error);
    assert!(error_chain.contains("Unclosed script tag") || error_chain.contains("unclosed"),
            "Expected unclosed tag error, got: {}", error_chain);
}

#[tokio::test]
async fn test_jsx_support() {
    let dir = TempDir::new().unwrap();
    let vue_content = r#"
<script lang="jsx">
export default {
  render() {
    return <div>JSX in Vue!</div>
  }
}
</script>
"#;

    let file_path = create_vue_file(&dir, "JsxComponent.vue", vue_content);

    let plugin = FobVuePlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Should be JSX module type
    assert!(matches!(
        output.module_type,
        Some(rolldown_common::ModuleType::Jsx)
    ));

    // Should contain JSX syntax
    assert!(output.code.contains("<div>"));
}

#[tokio::test]
async fn test_tsx_support() {
    let dir = TempDir::new().unwrap();
    let vue_content = r#"
<script setup lang="tsx">
import { ref } from 'vue'

const count = ref<number>(0)

const render = () => <div>Count: {count.value}</div>
</script>
"#;

    let file_path = create_vue_file(&dir, "TsxComponent.vue", vue_content);

    let plugin = FobVuePlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Should be TSX module type
    assert!(matches!(
        output.module_type,
        Some(rolldown_common::ModuleType::Tsx)
    ));

    // Should contain TypeScript and JSX
    assert!(output.code.contains("ref<number>"));
    assert!(output.code.contains("<div>"));
}

#[tokio::test]
async fn test_self_closing_script_tag() {
    let dir = TempDir::new().unwrap();
    let vue_content = r#"
<template>
  <div>External script</div>
</template>

<script src="./external.js" />
"#;

    let file_path = create_vue_file(&dir, "ExternalScript.vue", vue_content);

    let plugin = FobVuePlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Self-closing tags have empty content
    // Since there's no inline script, should return empty export
    assert_eq!(output.code.to_string(), "");
}

#[tokio::test]
async fn test_script_with_comments() {
    let dir = TempDir::new().unwrap();
    let vue_content = r#"
<script>
// This is a comment
export default {
  name: 'Commented',
  /* Multi-line
     comment */
  data() {
    return { x: 1 }
  }
}
</script>
"#;

    let file_path = create_vue_file(&dir, "Commented.vue", vue_content);

    let plugin = FobVuePlugin::new();
    let ctx = rolldown_plugin::PluginContext::new_napi_context();
    let args = HookLoadArgs { id: &file_path };

    let result = plugin.load(&ctx, &args).await.unwrap();

    assert!(result.is_some());
    let output = result.unwrap();

    // Should preserve comments
    assert!(output.code.contains("// This is a comment"));
    assert!(output.code.contains("/* Multi-line"));
    assert!(output.code.contains("export default"));
}

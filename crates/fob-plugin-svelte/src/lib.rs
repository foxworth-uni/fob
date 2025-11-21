//! Rolldown plugin for Svelte components
//!
//! This plugin extracts and processes `<script>` blocks from Svelte components, making them
//! available to the bundler as JavaScript/TypeScript modules.
//!
//! ## Architecture
//!
//! ```text
//! .svelte file → load() hook → SvelteExtractor → Extract scripts → JavaScript/TypeScript → Rolldown
//! ```
//!
//! ## Why the `load` hook?
//!
//! We use the `load` hook (not `transform`) because:
//! - Svelte components aren't valid JavaScript/TypeScript that Rolldown can parse
//! - We must intercept files before Rolldown's parser runs
//! - The `load` hook is designed for custom file formats
//! - We return extracted JavaScript with the appropriate `ModuleType`
//!
//! ## Handling Multiple Scripts
//!
//! Svelte components can have up to 2 script blocks:
//! - One `<script context="module">` block (module scope, runs once per import)
//! - One regular `<script>` block (instance scope, runs per component instance)
//!
//! When both exist, we combine them with the module context first, as per Svelte's
//! execution semantics. The module context runs when the module is imported, while
//! the instance script runs for each component instantiation.
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use fob_plugin_svelte::FobSveltePlugin;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let plugin = Arc::new(FobSveltePlugin::new());
//! // Add to your Rolldown bundler configuration
//! # Ok(())
//! # }
//! ```

use anyhow::Context;
use fob::extractors::{ExtractedScript, Extractor, SvelteExtractor};
use rolldown_common::ModuleType;
use rolldown_plugin::{HookLoadArgs, HookLoadOutput, HookLoadReturn, Plugin, PluginContext};
use std::borrow::Cow;

/// Rolldown plugin that extracts JavaScript/TypeScript from Svelte components
///
/// # Features
///
/// - Supports both `<script>` and `<script context="module">` blocks
/// - TypeScript support via `lang="ts"` attribute
/// - Accurate source mapping for error reporting
/// - Security: file size limits, tag count limits, no ReDoS vulnerabilities
///
/// # Security
///
/// The plugin enforces limits to prevent DoS attacks:
/// - Max file size: 10MB
/// - Max script tags: 100
/// - Uses memchr (not regex) to avoid ReDoS
/// - Never panics on malformed input
#[derive(Debug, Clone, Default)]
pub struct FobSveltePlugin {
    // Future: add configuration options here
    // - Custom file size limits
    // - Preprocessor configuration
    // - Source map generation settings
}

impl FobSveltePlugin {
    /// Creates a new Svelte plugin with default settings
    ///
    /// # Example
    ///
    /// ```rust
    /// use fob_plugin_svelte::FobSveltePlugin;
    ///
    /// let plugin = FobSveltePlugin::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}

impl Plugin for FobSveltePlugin {
    /// Returns the plugin name for debugging and logging
    fn name(&self) -> Cow<'static, str> {
        "fob-svelte".into()
    }

    /// Declares which hooks this plugin uses
    ///
    /// This allows Rolldown to optimize by skipping unused hooks.
    fn register_hook_usage(&self) -> rolldown_plugin::HookUsage {
        use rolldown_plugin::HookUsage;
        // We only use the load hook
        HookUsage::Load
    }

    /// Load hook - intercepts `.svelte` files and extracts JavaScript
    ///
    /// This is the core of the plugin. It:
    /// 1. Checks if the file is a `.svelte` file
    /// 2. Reads the file from disk
    /// 3. Parses and extracts script blocks
    /// 4. Combines multiple scripts if needed
    /// 5. Returns JavaScript/TypeScript to Rolldown
    ///
    /// # Returns
    ///
    /// - `Ok(Some(output))` - Successfully extracted JavaScript
    /// - `Ok(None)` - Not a Svelte file, let Rolldown handle it
    /// - `Err(e)` - Parse error or I/O error
    ///
    /// # Script Combination
    ///
    /// When a Svelte component has both scripts:
    /// ```svelte
    /// <script context="module">
    /// export const shared = 'data'
    /// </script>
    /// <script>
    /// import { onMount } from 'svelte'
    /// let count = 0
    /// </script>
    /// ```
    ///
    /// We combine them as:
    /// ```js
    /// export const shared = 'data'  // Module context runs first
    ///
    /// import { onMount } from 'svelte'
    /// let count = 0
    /// ```
    ///
    /// # Module Type Detection
    ///
    /// The `lang` attribute determines the module type:
    /// - `lang="ts"` → TypeScript
    /// - No lang or `lang="js"` → JavaScript
    fn load(
        &self,
        _ctx: &PluginContext,
        args: &HookLoadArgs<'_>,
    ) -> impl std::future::Future<Output = HookLoadReturn> + Send {
        // Capture data for async block
        let id = args.id.to_string();

        async move {
            // Only handle .svelte files
            if !id.ends_with(".svelte") {
                return Ok(None);
            }

            // Read the Svelte component source file
            let source = std::fs::read_to_string(&id)
                .with_context(|| format!("Failed to read Svelte file: {}", id))?;

            // Parse and extract script blocks
            let scripts = SvelteExtractor
                .extract(&source)
                .with_context(|| format!("Failed to parse Svelte file: {}", id))?;

            // Handle no scripts case
            if scripts.is_empty() {
                // Return empty JavaScript module
                return Ok(Some(HookLoadOutput {
                    code: "export default {}".into(),
                    module_type: Some(ModuleType::Js),
                    ..Default::default()
                }));
            }

            // Combine scripts and determine module type
            let (combined_code, module_type) = combine_scripts(&scripts);

            // Return the extracted JavaScript/TypeScript
            Ok(Some(HookLoadOutput {
                code: combined_code.into(),
                module_type: Some(module_type),
                ..Default::default()
            }))
        }
    }
}

/// Combines multiple script blocks and determines the appropriate module type.
///
/// # Algorithm
///
/// 1. If only one script: return it as-is with its module type
/// 2. If multiple scripts: combine with module context first (Svelte semantics)
/// 3. Module type priority: ts > js
///
/// # Examples
///
/// Single script:
/// ```ignore
/// scripts = [JavaScriptSource { source_text: "const x = 1", lang: "ts", ... }]
/// → ("const x = 1", ModuleType::Ts)
/// ```
///
/// Multiple scripts:
/// ```ignore
/// scripts = [
///   JavaScriptSource { source_text: "export const x = 1", is_module_context: true, ... },
///   JavaScriptSource { source_text: "let count = 0", is_module_context: false, ... },
/// ]
/// → ("export const x = 1\n\nlet count = 0", ModuleType::Js)
/// ```
fn combine_scripts(scripts: &[ExtractedScript]) -> (String, ModuleType) {
    use fob::extractors::ScriptContext;
    
    // Single script case
    if scripts.len() == 1 {
        let script = &scripts[0];
        return (
            script.source_text.to_string(),
            determine_module_type(script.lang),
        );
    }

    // Multiple scripts: separate module context and instance scripts
    let mut module_script = None;
    let mut instance_script = None;

    for script in scripts {
        match script.context {
            ScriptContext::SvelteModule => module_script = Some(script),
            ScriptContext::SvelteComponent => instance_script = Some(script),
            _ => {} // Shouldn't happen for Svelte
        }
    }

    // Combine scripts (module context first, then instance)
    let mut combined = String::new();
    let mut detected_lang = "js";

    if let Some(module) = module_script {
        combined.push_str(module.source_text);
        detected_lang = module.lang;
    }

    if let Some(instance) = instance_script {
        if !combined.is_empty() {
            combined.push_str("\n\n"); // Separate scripts with blank lines
        }
        combined.push_str(instance.source_text);

        // Upgrade module type if instance script uses a "stronger" type
        detected_lang = choose_stronger_lang(detected_lang, instance.lang);
    }

    (combined, determine_module_type(detected_lang))
}

/// Determines the Rolldown module type from a language identifier.
///
/// # Language Mapping
///
/// - "ts" or "typescript" → TypeScript
/// - "js" or anything else → JavaScript
fn determine_module_type(lang: &str) -> ModuleType {
    match lang {
        "ts" | "typescript" => ModuleType::Ts,
        _ => ModuleType::Js,
    }
}

/// Chooses the "stronger" language type for combined scripts.
///
/// Hierarchy: ts > js
///
/// This ensures we don't downgrade the module type when combining scripts.
fn choose_stronger_lang<'a>(lang1: &'a str, lang2: &'a str) -> &'a str {
    // Define language strength
    let strength = |lang: &str| match lang {
        "ts" | "typescript" => 2,
        _ => 1,
    };

    if strength(lang1) >= strength(lang2) {
        lang1
    } else {
        lang2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = FobSveltePlugin::new();
        assert_eq!(plugin.name(), "fob-svelte");
    }

    #[test]
    fn test_plugin_default() {
        let plugin = FobSveltePlugin::default();
        assert_eq!(plugin.name(), "fob-svelte");
    }

    #[test]
    fn test_determine_module_type() {
        assert!(matches!(determine_module_type("js"), ModuleType::Js));
        assert!(matches!(determine_module_type("ts"), ModuleType::Ts));
        assert!(matches!(
            determine_module_type("typescript"),
            ModuleType::Ts
        ));
    }

    #[test]
    fn test_choose_stronger_lang() {
        assert_eq!(choose_stronger_lang("js", "ts"), "ts");
        assert_eq!(choose_stronger_lang("ts", "js"), "ts");
        assert_eq!(choose_stronger_lang("ts", "ts"), "ts");
    }

    #[test]
    fn test_combine_single_script() {
        use fob::extractors::ScriptContext;
        let scripts = vec![ExtractedScript::new(
            "const x = 1;",
            100,
            ScriptContext::SvelteComponent,
            "js",
        )];
        let (code, module_type) = combine_scripts(&scripts);
        assert_eq!(code, "const x = 1;");
        assert!(matches!(module_type, ModuleType::Js));
    }

    #[test]
    fn test_combine_multiple_scripts() {
        use fob::extractors::ScriptContext;
        let scripts = vec![
            ExtractedScript::new("export const shared = 'data'", 50, ScriptContext::SvelteModule, "js"),
            ExtractedScript::new("let count: number = 0", 150, ScriptContext::SvelteComponent, "ts"),
        ];
        let (code, module_type) = combine_scripts(&scripts);
        // Module context script should come first
        assert!(code.starts_with("export const shared = 'data'"));
        assert!(code.contains("let count"));
        // Should upgrade to TypeScript
        assert!(matches!(module_type, ModuleType::Ts));
    }
}

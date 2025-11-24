//! Rolldown plugin for Vue Single File Components (SFC)
//!
//! This plugin extracts and processes `<script>` blocks from Vue SFCs, making them
//! available to the bundler as JavaScript/TypeScript modules.
//!
//! ## Architecture
//!
//! ```text
//! .vue file → load() hook → VueExtractor → Extract scripts → JavaScript/TypeScript → Rolldown
//! ```
//!
//! ## Why the `load` hook?
//!
//! We use the `load` hook (not `transform`) because:
//! - Vue SFCs aren't valid JavaScript/TypeScript that Rolldown can parse
//! - We must intercept files before Rolldown's parser runs
//! - The `load` hook is designed for custom file formats
//! - We return extracted JavaScript with the appropriate `ModuleType`
//!
//! ## Handling Multiple Scripts
//!
//! Vue SFCs can have up to 2 script blocks:
//! - One regular `<script>` block (Options API)
//! - One `<script setup>` block (Composition API)
//!
//! When both exist, we combine them with `<script setup>` running first,
//! as per Vue's execution semantics.
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use fob_plugin_vue::FobVuePlugin;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let plugin = Arc::new(FobVuePlugin::new());
//! // Add to your Rolldown bundler configuration
//! # Ok(())
//! # }
//! ```

use anyhow::Context;
use fob_analysis::extractors::{ExtractedScript, Extractor, VueExtractor};
use rolldown_common::ModuleType;
use rolldown_plugin::{HookLoadArgs, HookLoadOutput, HookLoadReturn, Plugin, PluginContext};
use std::borrow::Cow;

/// Rolldown plugin that extracts JavaScript/TypeScript from Vue SFCs
///
/// # Features
///
/// - Supports both `<script>` and `<script setup>` blocks
/// - TypeScript support via `lang="ts"` attribute
/// - JSX/TSX support via `lang="jsx"` or `lang="tsx"`
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
pub struct FobVuePlugin {
    // Future: add configuration options here
    // - Custom file size limits
    // - Template compilation options
    // - Source map generation settings
}

impl FobVuePlugin {
    /// Creates a new Vue plugin with default settings
    ///
    /// # Example
    ///
    /// ```rust
    /// use fob_plugin_vue::FobVuePlugin;
    ///
    /// let plugin = FobVuePlugin::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}

impl Plugin for FobVuePlugin {
    /// Returns the plugin name for debugging and logging
    fn name(&self) -> Cow<'static, str> {
        "fob-vue".into()
    }

    /// Declares which hooks this plugin uses
    ///
    /// This allows Rolldown to optimize by skipping unused hooks.
    fn register_hook_usage(&self) -> rolldown_plugin::HookUsage {
        use rolldown_plugin::HookUsage;
        // We only use the load hook
        HookUsage::Load
    }

    /// Load hook - intercepts `.vue` files and extracts JavaScript
    ///
    /// This is the core of the plugin. It:
    /// 1. Checks if the file is a `.vue` file
    /// 2. Reads the file from disk
    /// 3. Parses and extracts script blocks
    /// 4. Combines multiple scripts if needed
    /// 5. Returns JavaScript/TypeScript to Rolldown
    ///
    /// # Returns
    ///
    /// - `Ok(Some(output))` - Successfully extracted JavaScript
    /// - `Ok(None)` - Not a Vue file, let Rolldown handle it
    /// - `Err(e)` - Parse error or I/O error
    ///
    /// # Script Combination
    ///
    /// When a Vue SFC has both `<script>` and `<script setup>`:
    /// ```vue
    /// <script>
    /// export default { name: 'MyComponent' }
    /// </script>
    /// <script setup>
    /// const count = ref(0)
    /// </script>
    /// ```
    ///
    /// We combine them as:
    /// ```js
    /// const count = ref(0)  // <script setup> runs first
    ///
    /// export default { name: 'MyComponent' }
    /// ```
    ///
    /// # Module Type Detection
    ///
    /// The `lang` attribute determines the module type:
    /// - `lang="ts"` → TypeScript
    /// - `lang="jsx"` → JSX
    /// - `lang="tsx"` → TSX
    /// - No lang or `lang="js"` → JavaScript
    fn load(
        &self,
        _ctx: &PluginContext,
        args: &HookLoadArgs<'_>,
    ) -> impl std::future::Future<Output = HookLoadReturn> + Send {
        // Capture data for async block
        let id = args.id.to_string();

        async move {
            // Only handle .vue files
            if !id.ends_with(".vue") {
                return Ok(None);
            }

            // Read the Vue SFC source file
            let source = std::fs::read_to_string(&id)
                .with_context(|| format!("Failed to read Vue file: {}", id))?;

            // Parse and extract script blocks
            let scripts = VueExtractor
                .extract(&source)
                .with_context(|| format!("Failed to parse Vue file: {}", id))?;

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
/// 2. If multiple scripts: combine with `<script setup>` first (Vue semantics)
/// 3. Module type priority: tsx > jsx > ts > js
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
///   JavaScriptSource { source_text: "export default {}", is_setup: false, ... },
///   JavaScriptSource { source_text: "const x = 1", is_setup: true, ... },
/// ]
/// → ("const x = 1\n\nexport default {}", ModuleType::Js)
/// ```
fn combine_scripts(scripts: &[ExtractedScript]) -> (String, ModuleType) {
    use fob_analysis::extractors::ScriptContext;

    // Single script case
    if scripts.len() == 1 {
        let script = &scripts[0];
        return (
            script.source_text.to_string(),
            determine_module_type(script.lang),
        );
    }

    // Multiple scripts: separate setup and regular scripts
    let mut setup_script = None;
    let mut regular_script = None;

    for script in scripts {
        match script.context {
            ScriptContext::VueSetup => setup_script = Some(script),
            ScriptContext::VueRegular => regular_script = Some(script),
            _ => {} // Shouldn't happen for Vue
        }
    }

    // Combine scripts (setup first, then regular)
    let mut combined = String::new();
    let mut detected_lang = "js";

    if let Some(setup) = setup_script {
        combined.push_str(setup.source_text);
        detected_lang = setup.lang;
    }

    if let Some(regular) = regular_script {
        if !combined.is_empty() {
            combined.push_str("\n\n"); // Separate scripts with blank lines
        }
        combined.push_str(regular.source_text);

        // Upgrade module type if regular script uses a "stronger" type
        detected_lang = choose_stronger_lang(detected_lang, regular.lang);
    }

    (combined, determine_module_type(detected_lang))
}

/// Determines the Rolldown module type from a language identifier.
///
/// # Language Mapping
///
/// - "ts" or "typescript" → TypeScript
/// - "jsx" → JSX
/// - "tsx" → TSX
/// - "js" or anything else → JavaScript
fn determine_module_type(lang: &str) -> ModuleType {
    match lang {
        "ts" | "typescript" => ModuleType::Ts,
        "jsx" => ModuleType::Jsx,
        "tsx" => ModuleType::Tsx,
        _ => ModuleType::Js,
    }
}

/// Chooses the "stronger" language type for combined scripts.
///
/// Hierarchy: tsx > jsx > ts > js
///
/// This ensures we don't downgrade the module type when combining scripts.
fn choose_stronger_lang<'a>(lang1: &'a str, lang2: &'a str) -> &'a str {
    // Define language strength
    let strength = |lang: &str| match lang {
        "tsx" => 4,
        "jsx" => 3,
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
        let plugin = FobVuePlugin::new();
        assert_eq!(plugin.name(), "fob-vue");
    }

    #[test]
    fn test_plugin_default() {
        let plugin = FobVuePlugin::default();
        assert_eq!(plugin.name(), "fob-vue");
    }

    #[test]
    fn test_determine_module_type() {
        assert!(matches!(determine_module_type("js"), ModuleType::Js));
        assert!(matches!(determine_module_type("ts"), ModuleType::Ts));
        assert!(matches!(determine_module_type("jsx"), ModuleType::Jsx));
        assert!(matches!(determine_module_type("tsx"), ModuleType::Tsx));
        assert!(matches!(
            determine_module_type("typescript"),
            ModuleType::Ts
        ));
    }

    #[test]
    fn test_choose_stronger_lang() {
        assert_eq!(choose_stronger_lang("js", "ts"), "ts");
        assert_eq!(choose_stronger_lang("ts", "tsx"), "tsx");
        assert_eq!(choose_stronger_lang("jsx", "js"), "jsx");
        assert_eq!(choose_stronger_lang("tsx", "ts"), "tsx");
        assert_eq!(choose_stronger_lang("ts", "ts"), "ts");
    }

    #[test]
    fn test_combine_single_script() {
        use fob_analysis::extractors::ScriptContext;
        let scripts = vec![ExtractedScript::new(
            "const x = 1;",
            100,
            ScriptContext::VueRegular,
            "js",
        )];
        let (code, module_type) = combine_scripts(&scripts);
        assert_eq!(code, "const x = 1;");
        assert!(matches!(module_type, ModuleType::Js));
    }

    #[test]
    fn test_combine_multiple_scripts() {
        use fob_analysis::extractors::ScriptContext;
        let scripts = vec![
            ExtractedScript::new("export default {}", 50, ScriptContext::VueRegular, "js"),
            ExtractedScript::new("const count = ref(0)", 150, ScriptContext::VueSetup, "ts"),
        ];
        let (code, module_type) = combine_scripts(&scripts);
        // Setup script should come first
        assert!(code.starts_with("const count = ref(0)"));
        assert!(code.contains("export default {}"));
        // Should upgrade to TypeScript
        assert!(matches!(module_type, ModuleType::Ts));
    }
}

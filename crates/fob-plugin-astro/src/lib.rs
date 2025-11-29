//! Rolldown plugin for Astro components
//!
//! This plugin extracts and processes frontmatter and `<script>` blocks from Astro
//! components, making them available to the bundler as JavaScript/TypeScript modules.
//!
//! ## Architecture
//!
//! ```text
//! .astro file → load() hook → AstroExtractor → Extract frontmatter + scripts → JS/TS → Rolldown
//! ```
//!
//! ## Why the `load` hook?
//!
//! We use the `load` hook (not `transform`) because:
//! - Astro components aren't valid JavaScript/TypeScript that Rolldown can parse
//! - We must intercept files before Rolldown's parser runs
//! - The `load` hook is designed for custom file formats
//! - We return extracted JavaScript with the appropriate `ModuleType`
//!
//! ## Handling Frontmatter and Scripts
//!
//! Astro components can contain:
//! - One frontmatter block at the start (delimited by `---`, TypeScript by default)
//! - Multiple `<script>` tags in the template (JavaScript by default)
//!
//! Frontmatter runs on the server during build/SSR, while script tags run in the browser.
//! When both exist, we combine them with frontmatter first, as it executes during the
//! component's module loading phase.
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use fob_plugin_astro::FobAstroPlugin;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let plugin = Arc::new(FobAstroPlugin::new());
//! // Add to your Rolldown bundler configuration
//! # Ok(())
//! # }
//! ```

use anyhow::Context;
use fob_analysis::extractors::{AstroExtractor, ExtractedScript, Extractor};
use fob_bundler::{
    HookLoadArgs, HookLoadOutput, HookLoadReturn, ModuleType, Plugin, PluginContext,
};
use std::borrow::Cow;

/// Rolldown plugin that extracts JavaScript/TypeScript from Astro components
///
/// # Features
///
/// - Supports frontmatter (delimited by `---` at file start)
/// - Supports multiple `<script>` tags in the template
/// - Frontmatter is TypeScript by default
/// - Script tags are JavaScript by default
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
pub struct FobAstroPlugin {
    // Future: add configuration options here
    // - Custom file size limits
    // - Preprocessor configuration
    // - Source map generation settings
}

impl FobAstroPlugin {
    /// Creates a new Astro plugin with default settings
    ///
    /// # Example
    ///
    /// ```rust
    /// use fob_plugin_astro::FobAstroPlugin;
    ///
    /// let plugin = FobAstroPlugin::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}

impl Plugin for FobAstroPlugin {
    /// Returns the plugin name for debugging and logging
    fn name(&self) -> Cow<'static, str> {
        "fob-astro".into()
    }

    /// Declares which hooks this plugin uses
    ///
    /// This allows Rolldown to optimize by skipping unused hooks.
    fn register_hook_usage(&self) -> fob_bundler::HookUsage {
        use fob_bundler::HookUsage;
        // We only use the load hook
        HookUsage::Load
    }

    /// Load hook - intercepts `.astro` files and extracts JavaScript
    ///
    /// This is the core of the plugin. It:
    /// 1. Checks if the file is a `.astro` file
    /// 2. Reads the file from disk
    /// 3. Parses and extracts frontmatter and script blocks
    /// 4. Combines multiple scripts if needed
    /// 5. Returns JavaScript/TypeScript to Rolldown
    ///
    /// # Returns
    ///
    /// - `Ok(Some(output))` - Successfully extracted JavaScript
    /// - `Ok(None)` - Not an Astro file, let Rolldown handle it
    /// - `Err(e)` - Parse error or I/O error
    ///
    /// # Script Combination
    ///
    /// When an Astro component has frontmatter and scripts:
    /// ```astro
    /// ---
    /// const title = 'My Page'
    /// const data = await fetchData()
    /// ---
    /// <html>
    ///   <head><title>{title}</title></head>
    ///   <body>
    ///     <script>
    ///       console.log('Client-side code')
    ///     </script>
    ///   </body>
    /// </html>
    /// ```
    ///
    /// We combine them as:
    /// ```js
    /// const title = 'My Page'  // Frontmatter runs first (server-side)
    /// const data = await fetchData()
    ///
    /// console.log('Client-side code')  // Scripts run in browser
    /// ```
    ///
    /// # Module Type Detection
    ///
    /// - Frontmatter is always TypeScript
    /// - Scripts are JavaScript by default
    /// - If any source is TypeScript, the combined output is TypeScript
    fn load(
        &self,
        _ctx: &PluginContext,
        args: &HookLoadArgs<'_>,
    ) -> impl std::future::Future<Output = HookLoadReturn> + Send {
        // Capture data for async block
        let id = args.id.to_string();

        async move {
            // Only handle .astro files
            if !id.ends_with(".astro") {
                return Ok(None);
            }

            // Read the Astro component source file
            let source = std::fs::read_to_string(&id)
                .with_context(|| format!("Failed to read Astro file: {}", id))?;

            // Parse and extract frontmatter and script blocks
            let scripts = AstroExtractor
                .extract(&source)
                .with_context(|| format!("Failed to parse Astro file: {}", id))?;

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
/// 2. If multiple scripts: combine with frontmatter first, then scripts in order
/// 3. Module type priority: ts > js (if any source is TS, output is TS)
///
/// # Examples
///
/// Single frontmatter:
/// ```ignore
/// scripts = [JavaScriptSource { source_text: "const x = 1", is_frontmatter: true, lang: "ts" }]
/// → ("const x = 1", ModuleType::Ts)
/// ```
///
/// Multiple scripts:
/// ```ignore
/// scripts = [
///   JavaScriptSource { source_text: "const x = 1", is_frontmatter: true, lang: "ts" },
///   JavaScriptSource { source_text: "console.log(x)", is_frontmatter: false, lang: "js" },
/// ]
/// → ("const x = 1\n\nconsole.log(x)", ModuleType::Ts)
/// ```
fn combine_scripts(scripts: &[ExtractedScript]) -> (String, ModuleType) {
    // Single script case
    if scripts.len() == 1 {
        let script = &scripts[0];
        return (
            script.source_text.to_string(),
            determine_module_type(script.lang),
        );
    }

    // Multiple scripts: combine frontmatter first, then scripts in order
    let mut combined = String::new();
    let mut has_typescript = false;

    for (i, script) in scripts.iter().enumerate() {
        if i > 0 && !combined.is_empty() {
            combined.push_str("\n\n"); // Separate scripts with blank lines
        }

        combined.push_str(script.source_text);

        // Track if any script is TypeScript
        if script.lang == "ts" || script.lang == "typescript" {
            has_typescript = true;
        }
    }

    let module_type = if has_typescript {
        ModuleType::Ts
    } else {
        ModuleType::Js
    };

    (combined, module_type)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = FobAstroPlugin::new();
        assert_eq!(plugin.name(), "fob-astro");
    }

    #[test]
    fn test_plugin_default() {
        let plugin = FobAstroPlugin::default();
        assert_eq!(plugin.name(), "fob-astro");
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
    fn test_combine_single_script() {
        use fob_analysis::extractors::ScriptContext;
        let scripts = vec![ExtractedScript::new(
            "const x = 1;",
            4,
            ScriptContext::AstroFrontmatter,
            "ts",
        )];
        let (code, module_type) = combine_scripts(&scripts);
        assert_eq!(code, "const x = 1;");
        assert!(matches!(module_type, ModuleType::Ts));
    }

    #[test]
    fn test_combine_multiple_scripts() {
        use fob_analysis::extractors::ScriptContext;
        let scripts = vec![
            ExtractedScript::new(
                "const title = 'Page'",
                4,
                ScriptContext::AstroFrontmatter,
                "ts",
            ),
            ExtractedScript::new("console.log(title)", 100, ScriptContext::AstroScript, "js"),
            ExtractedScript::new("alert('hello')", 200, ScriptContext::AstroScript, "js"),
        ];
        let (code, module_type) = combine_scripts(&scripts);
        // Frontmatter should come first
        assert!(code.starts_with("const title = 'Page'"));
        assert!(code.contains("console.log(title)"));
        assert!(code.contains("alert('hello')"));
        // Should be TypeScript (frontmatter is TS)
        assert!(matches!(module_type, ModuleType::Ts));
    }

    #[test]
    fn test_combine_only_scripts() {
        use fob_analysis::extractors::ScriptContext;
        let scripts = vec![
            ExtractedScript::new("console.log('a')", 50, ScriptContext::AstroScript, "js"),
            ExtractedScript::new("console.log('b')", 100, ScriptContext::AstroScript, "js"),
        ];
        let (code, module_type) = combine_scripts(&scripts);
        assert!(code.contains("console.log('a')"));
        assert!(code.contains("console.log('b')"));
        assert!(matches!(module_type, ModuleType::Js));
    }
}

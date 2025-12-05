//! Runtime file plugin for serving virtual files to Rolldown
//!
//! This plugin uses BundlerRuntime as the single source of truth for file access.
//! It serves virtual files via Rolldown hooks while delegating to Runtime for
//! all file operations. Runs in the Virtual phase to ensure virtual files are
//! available before other plugins try to resolve or load them.

use crate::Runtime;
use crate::plugins::{FobPlugin, PluginPhase};
use crate::runtime::BundlerRuntime;
use anyhow::Context;
use rolldown_common::{ModuleType, ResolvedExternal};
use rolldown_plugin::{
    HookLoadArgs, HookLoadOutput, HookLoadReturn, HookResolveIdArgs, HookResolveIdOutput,
    HookResolveIdReturn, Plugin, PluginContext,
};
use std::borrow::Cow;
use std::path::Path;
use std::sync::Arc;

/// Plugin that serves virtual files from BundlerRuntime to Rolldown
///
/// This plugin implements the `resolve_id` and `load` hooks to serve virtual
/// files that don't exist on disk. It delegates all file operations to
/// BundlerRuntime, which checks virtual files first, then falls back to
/// the filesystem.
#[derive(Debug, Clone)]
pub struct RuntimeFilePlugin {
    /// Runtime that manages virtual files and filesystem access
    runtime: Arc<BundlerRuntime>,
}

impl RuntimeFilePlugin {
    /// Create a new RuntimeFilePlugin with the given runtime
    pub fn new(runtime: Arc<BundlerRuntime>) -> Self {
        Self { runtime }
    }
}

impl Plugin for RuntimeFilePlugin {
    fn name(&self) -> Cow<'static, str> {
        "fob-runtime-files".into()
    }

    fn register_hook_usage(&self) -> rolldown_plugin::HookUsage {
        use rolldown_plugin::HookUsage;
        HookUsage::ResolveId | HookUsage::Load
    }

    /// Resolve ID hook - confirms we can serve this file
    ///
    /// This hook checks if Runtime knows about this file (virtual or real).
    /// It also handles extensionless imports by trying common extensions.
    fn resolve_id(
        &self,
        _ctx: &PluginContext,
        args: &HookResolveIdArgs,
    ) -> impl std::future::Future<Output = HookResolveIdReturn> + Send {
        let specifier = args.specifier.to_string();
        let runtime = Arc::clone(&self.runtime);

        async move {
            let path = Path::new(&specifier);

            // Only claim virtual files - let Rolldown handle real files
            if runtime.has_virtual_file(path) {
                return Ok(Some(HookResolveIdOutput {
                    id: specifier.into(),
                    external: Some(ResolvedExternal::Bool(false)),
                    ..Default::default()
                }));
            }

            // Try with extensions for extensionless virtual imports
            for ext in &[".tsx", ".ts", ".jsx", ".js"] {
                let with_ext = format!("{}{}", specifier, ext);
                let path = Path::new(&with_ext);
                if runtime.has_virtual_file(path) {
                    return Ok(Some(HookResolveIdOutput {
                        id: with_ext.into(),
                        external: Some(ResolvedExternal::Bool(false)),
                        ..Default::default()
                    }));
                }
            }

            // Not a virtual file, let Rolldown handle it (including real files)
            Ok(None)
        }
    }

    /// Load hook - serves content from Runtime
    ///
    /// Only serves virtual files (real files are handled by Rolldown's
    /// default loader). This ensures virtual files are transparent to
    /// other plugins.
    fn load(
        &self,
        _ctx: &PluginContext,
        args: &HookLoadArgs<'_>,
    ) -> impl std::future::Future<Output = HookLoadReturn> + Send {
        let id = args.id.to_string();
        let runtime = Arc::clone(&self.runtime);

        async move {
            let path = Path::new(&id);

            // Only serve if it's a virtual file (real files handled by Rolldown)
            if !runtime.has_virtual_file(path) {
                return Ok(None);
            }

            // Skip .mdx files - let the MDX plugin transform them
            // The MDX plugin will use Runtime::read_file() to get the content
            if id.ends_with(".mdx") {
                return Ok(None);
            }

            // Read content from Runtime
            let content = Runtime::read_file(&*runtime, path)
                .await
                .with_context(|| format!("Failed to read virtual file: {}", id))?;

            let source = String::from_utf8(content)
                .with_context(|| format!("Virtual file {} contains invalid UTF-8", id))?;

            Ok(Some(HookLoadOutput {
                code: source.into(),
                module_type: Some(infer_module_type(&id)),
                ..Default::default()
            }))
        }
    }
}

impl FobPlugin for RuntimeFilePlugin {
    fn phase(&self) -> PluginPhase {
        PluginPhase::Virtual
    }
}

/// Infers module type from file extension
fn infer_module_type(id: &str) -> ModuleType {
    match Path::new(id).extension().and_then(|e| e.to_str()) {
        Some("tsx") => ModuleType::Tsx,
        Some("ts") => ModuleType::Ts,
        Some("jsx") => ModuleType::Jsx,
        Some("mdx") => ModuleType::Jsx, // MDX compiles to JSX
        Some("css") => ModuleType::Css,
        Some("json") => ModuleType::Json,
        Some("js") | Some("mjs") | Some("cjs") => ModuleType::Js,
        _ => ModuleType::Js, // Default to JavaScript
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_module_type() {
        assert!(matches!(infer_module_type("file.js"), ModuleType::Js));
        assert!(matches!(infer_module_type("file.jsx"), ModuleType::Jsx));
        assert!(matches!(infer_module_type("file.ts"), ModuleType::Ts));
        assert!(matches!(infer_module_type("file.tsx"), ModuleType::Tsx));
        assert!(matches!(infer_module_type("file.mdx"), ModuleType::Jsx));
        assert!(matches!(infer_module_type("file.css"), ModuleType::Css));
        assert!(matches!(infer_module_type("file.json"), ModuleType::Json));
    }
}

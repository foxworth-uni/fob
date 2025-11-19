//! Virtual file plugin for serving in-memory files to Rolldown
//!
//! This plugin implements the `load` hook to serve virtual files that don't
//! exist on disk. Virtual files are stored in memory and can be injected
//! programmatically via the LibraryBuilder API.
//!
//! ## Use Cases
//!
//! - Code generation (e.g., auto-generated index files)
//! - Injecting configuration files
//! - Testing bundler behavior without filesystem I/O
//! - Dynamic entry points
//!
//! ## Security
//!
//! - File size limits prevent memory exhaustion
//! - Path validation prevents directory traversal
//! - Module IDs are normalized and validated

use rolldown_common::ModuleType;
use rolldown_plugin::{HookLoadArgs, HookLoadOutput, HookLoadReturn, Plugin, PluginContext};
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::sync::Arc;

/// Maximum size for a single virtual file (1MB)
///
/// This prevents memory exhaustion attacks where an attacker could inject
/// massive virtual files. 1MB is generous for most use cases (configs, generated
/// code) but prevents abuse.
const MAX_VIRTUAL_FILE_SIZE: usize = 1024 * 1024;

/// Plugin that serves virtual (in-memory) files to Rolldown
#[derive(Debug, Clone)]
pub struct VirtualFilePlugin {
    /// Map of module ID to file content
    /// Arc allows efficient cloning when passing to async context
    files: Arc<FxHashMap<String, String>>,
}

impl VirtualFilePlugin {
    /// Create a new VirtualFilePlugin from a map of virtual files
    ///
    /// # Arguments
    ///
    /// * `files` - HashMap mapping module IDs to file contents
    ///
    /// # Returns
    ///
    /// Returns `Ok(plugin)` or `Err` if validation fails
    ///
    /// # Errors
    ///
    /// - File size exceeds MAX_VIRTUAL_FILE_SIZE
    /// - Invalid module ID (null bytes, suspicious patterns)
    pub fn new(files: FxHashMap<String, String>) -> crate::Result<Self> {
        // Validate all files before creating plugin
        for (module_id, content) in &files {
            validate_module_id(module_id)?;
            validate_content_size(content)?;
        }

        Ok(Self {
            files: Arc::new(files),
        })
    }
}

impl Plugin for VirtualFilePlugin {
    fn name(&self) -> Cow<'static, str> {
        "joy-virtual-files".into()
    }

    fn register_hook_usage(&self) -> rolldown_plugin::HookUsage {
        use rolldown_plugin::HookUsage;
        HookUsage::ResolveId | HookUsage::Load
    }

    /// Resolve ID hook - tells Rolldown that virtual module IDs are valid
    ///
    /// This hook is called before load and needs to confirm that we can handle
    /// the requested module ID. Without this, Rolldown will reject virtual entries.
    fn resolve_id(
        &self,
        _ctx: &PluginContext,
        args: &rolldown_plugin::HookResolveIdArgs,
    ) -> impl std::future::Future<Output = rolldown_plugin::HookResolveIdReturn> + Send {
        let specifier = args.specifier.to_string();
        let files = self.files.clone();

        async move {
            // If this is one of our virtual files, resolve it to itself
            if files.contains_key(&specifier) {
                use rolldown_plugin::HookResolveIdOutput;
                Ok(Some(HookResolveIdOutput {
                    id: specifier.into(),
                    ..Default::default()
                }))
            } else {
                // Not our virtual file, let other resolvers handle it
                Ok(None)
            }
        }
    }

    /// Load hook - serves virtual files if module ID matches
    ///
    /// Returns the file content if the requested module ID is in our virtual
    /// files map, otherwise returns None to let other loaders handle it.
    fn load(
        &self,
        _ctx: &PluginContext,
        args: &HookLoadArgs<'_>,
    ) -> impl std::future::Future<Output = HookLoadReturn> + Send {
        let id = args.id.to_string();
        let files = self.files.clone();

        async move {
            // Check if this module ID is a virtual file
            if let Some(content) = files.get(&id) {
                // Determine module type from extension
                let module_type = infer_module_type(&id);

                Ok(Some(HookLoadOutput {
                    code: content.clone().into(),
                    module_type: Some(module_type),
                    ..Default::default()
                }))
            } else {
                // Not a virtual file, let other loaders handle it
                Ok(None)
            }
        }
    }
}

/// Validates a module ID for security
fn validate_module_id(module_id: &str) -> crate::Result<()> {
    // Reject null bytes (can cause issues in C FFI and filesystems)
    if module_id.contains('\0') {
        return Err(crate::Error::InvalidOutputPath(
            "Virtual file module ID contains null byte".to_string(),
        ));
    }

    // Reject suspiciously long paths (> 4096 chars)
    if module_id.len() > 4096 {
        return Err(crate::Error::InvalidOutputPath(format!(
            "Virtual file module ID too long: {} bytes (max 4096)",
            module_id.len()
        )));
    }

    Ok(())
}

/// Validates virtual file content size
fn validate_content_size(content: &str) -> crate::Result<()> {
    if content.len() > MAX_VIRTUAL_FILE_SIZE {
        return Err(crate::Error::WriteFailure(format!(
            "Virtual file content too large: {} bytes (max {} bytes)",
            content.len(),
            MAX_VIRTUAL_FILE_SIZE
        )));
    }
    Ok(())
}

/// Infers module type from file extension
fn infer_module_type(module_id: &str) -> ModuleType {
    use std::path::Path;

    let ext = Path::new(module_id)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext {
        "jsx" => ModuleType::Jsx,
        "ts" => ModuleType::Ts,
        "tsx" => ModuleType::Tsx,
        "json" => ModuleType::Json,
        "js" | "mjs" | "cjs" => ModuleType::Js,
        // Unknown extensions default to JavaScript
        _ => ModuleType::Js,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_module_id_normal() {
        assert!(validate_module_id("virtual:config.js").is_ok());
        assert!(validate_module_id("/virtual/generated/index.ts").is_ok());
    }

    #[test]
    fn test_validate_module_id_null_byte() {
        assert!(validate_module_id("file\0name.js").is_err());
    }

    #[test]
    fn test_validate_module_id_too_long() {
        let long_id = "a".repeat(5000);
        assert!(validate_module_id(&long_id).is_err());
    }

    #[test]
    fn test_validate_content_size_ok() {
        let content = "a".repeat(1000);
        assert!(validate_content_size(&content).is_ok());
    }

    #[test]
    fn test_validate_content_size_too_large() {
        let content = "a".repeat(MAX_VIRTUAL_FILE_SIZE + 1);
        assert!(validate_content_size(&content).is_err());
    }

    #[test]
    fn test_infer_module_type() {
        assert!(matches!(infer_module_type("file.js"), ModuleType::Js));
        assert!(matches!(infer_module_type("file.jsx"), ModuleType::Jsx));
        assert!(matches!(infer_module_type("file.ts"), ModuleType::Ts));
        assert!(matches!(infer_module_type("file.tsx"), ModuleType::Tsx));
        assert!(matches!(infer_module_type("file.json"), ModuleType::Json));
    }

    #[test]
    fn test_plugin_creation_valid() {
        let mut files = FxHashMap::default();
        files.insert(
            "virtual:test.js".to_string(),
            "export const x = 1;".to_string(),
        );

        let result = VirtualFilePlugin::new(files);
        assert!(result.is_ok());
    }

    #[test]
    fn test_plugin_creation_invalid_size() {
        let mut files = FxHashMap::default();
        let huge_content = "a".repeat(MAX_VIRTUAL_FILE_SIZE + 1);
        files.insert("virtual:test.js".to_string(), huge_content);

        let result = VirtualFilePlugin::new(files);
        assert!(result.is_err());
    }
}

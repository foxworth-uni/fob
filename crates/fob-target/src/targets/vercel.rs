//! Vercel Node.js deployment target.

use crate::target::DeploymentTarget;
use fob_bundler::{ExportConditions, NodeBuiltins, RuntimeEnvironment};
use std::path::Path;

/// Vercel Node.js deployment target
///
/// This target configures builds for Vercel's Node.js runtime:
/// - Uses Node.js export conditions (not browser)
/// - Externalizes Node.js built-ins
/// - Generates `.vc-config.json` if needed
pub struct VercelNodeTarget;

impl DeploymentTarget for VercelNodeTarget {
    fn name(&self) -> &'static str {
        "vercel-node"
    }

    fn runtime(&self) -> RuntimeEnvironment {
        RuntimeEnvironment::Node
    }

    fn conditions(&self) -> ExportConditions {
        ExportConditions::node()
    }

    fn node_builtins(&self) -> NodeBuiltins {
        NodeBuiltins::External
    }

    fn generate_output(
        &self,
        _build_result: &fob_bundler::BuildResult,
        output_dir: &Path,
    ) -> fob_bundler::Result<()> {
        // Generate package.json with "type": "module" if it doesn't exist
        let package_json_path = output_dir.join("package.json");
        if !package_json_path.exists() {
            let package_json = serde_json::json!({
                "type": "module",
                "version": "1.0.0"
            });
            let json_str = serde_json::to_string_pretty(&package_json)
                .map_err(|e| fob_bundler::Error::InvalidConfig(format!("JSON error: {}", e)))?;
            std::fs::write(&package_json_path, json_str)?;
        }

        // Note: .vc-config.json is optional and typically auto-detected by Vercel
        // We don't generate it here to avoid conflicts with user configuration

        Ok(())
    }
}

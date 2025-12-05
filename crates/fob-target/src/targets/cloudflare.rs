//! Cloudflare Workers deployment target.

use crate::target::DeploymentTarget;
use fob_bundler::{ExportConditions, NodeBuiltins, RuntimeEnvironment};
use std::path::Path;

/// Cloudflare Workers deployment target
///
/// This target configures builds for Cloudflare Workers:
/// - Uses edge worker export conditions
/// - Errors on Node.js built-in imports
/// - Can generate wrangler.toml or _routes.json if needed
pub struct CloudflareWorkersTarget;

impl DeploymentTarget for CloudflareWorkersTarget {
    fn name(&self) -> &'static str {
        "cloudflare-workers"
    }

    fn runtime(&self) -> RuntimeEnvironment {
        RuntimeEnvironment::EdgeWorker
    }

    fn conditions(&self) -> ExportConditions {
        ExportConditions::edge()
    }

    fn node_builtins(&self) -> NodeBuiltins {
        NodeBuiltins::Error
    }

    fn generate_output(
        &self,
        _build_result: &fob_bundler::BuildResult,
        _output_dir: &Path,
    ) -> fob_bundler::Result<()> {
        // Note: wrangler.toml and _routes.json are typically managed by the user
        // or by Cloudflare's tooling. We don't generate them here to avoid conflicts.
        Ok(())
    }
}

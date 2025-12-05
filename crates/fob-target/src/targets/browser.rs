//! Browser deployment target.

use crate::target::DeploymentTarget;
use fob_bundler::{ExportConditions, NodeBuiltins, RuntimeEnvironment};
use std::path::Path;

/// Browser deployment target
///
/// This target configures builds for browser SPAs:
/// - Uses browser export conditions
/// - Errors on Node.js built-in imports
/// - No special output generation needed
pub struct BrowserTarget;

impl DeploymentTarget for BrowserTarget {
    fn name(&self) -> &'static str {
        "browser"
    }

    fn runtime(&self) -> RuntimeEnvironment {
        RuntimeEnvironment::Browser
    }

    fn conditions(&self) -> ExportConditions {
        ExportConditions::browser()
    }

    fn node_builtins(&self) -> NodeBuiltins {
        NodeBuiltins::Error
    }

    fn generate_output(
        &self,
        _build_result: &fob_bundler::BuildResult,
        _output_dir: &Path,
    ) -> fob_bundler::Result<()> {
        // Browser targets just write files to output directory
        // No special configuration needed
        Ok(())
    }
}

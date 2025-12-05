//! Deployment target types and configuration.
//!
//! This module defines the core types for deployment targets:
//! - `RuntimeEnvironment`: Where code will execute
//! - `ExportConditions`: Module resolution conditions
//! - `NodeBuiltins`: How to handle Node.js built-in modules
//! - `DeploymentTarget`: Trait for deployment target adapters

use std::path::Path;

/// Environment where code will execute
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeEnvironment {
    /// Node.js (full APIs)
    Node,
    /// V8 isolate (Cloudflare Workers, Deno Deploy)
    EdgeWorker,
    /// Browser
    Browser,
}

/// Export conditions for module resolution
///
/// This enum provides zero-allocation construction for common condition sets.
/// Allocation only happens when converting to `Vec<String>` via `to_vec()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportConditions {
    /// Node.js runtime conditions: `["node", "import", "module", "default"]`
    Node,
    /// Edge worker conditions: `["edge-light", "workerd", "worker", "browser", "import", "default"]`
    Edge,
    /// Browser runtime conditions: `["browser", "import", "module", "default"]`
    Browser,
}

impl ExportConditions {
    /// Export conditions for Node.js runtime
    #[inline]
    pub fn node() -> Self {
        Self::Node
    }

    /// Export conditions for edge workers (Cloudflare, Deno Deploy)
    ///
    /// Order matters: more specific conditions come first.
    /// - `edge-light`: Vercel Edge Functions (most specific)
    /// - `workerd`: Cloudflare Workers runtime
    /// - `worker`: Generic web worker
    /// - `browser`: Browser-compatible fallback
    #[inline]
    pub fn edge() -> Self {
        Self::Edge
    }

    /// Export conditions for browser runtime
    #[inline]
    pub fn browser() -> Self {
        Self::Browser
    }

    /// Get the condition names as a static slice (zero allocation)
    pub fn as_slice(&self) -> &'static [&'static str] {
        match self {
            Self::Node => &["node", "import", "module", "default"],
            Self::Edge => &[
                "edge-light",
                "workerd",
                "worker",
                "browser",
                "import",
                "default",
            ],
            Self::Browser => &["browser", "import", "module", "default"],
        }
    }

    /// Convert to a Vec<String> for Rolldown compatibility
    ///
    /// This allocates - use `as_slice()` or `contains()` when possible.
    pub fn to_vec(&self) -> Vec<String> {
        self.as_slice().iter().map(|s| (*s).to_string()).collect()
    }

    /// Check if this condition set contains a specific condition name
    ///
    /// Zero allocation - uses static slice comparison.
    pub fn contains(&self, name: &str) -> bool {
        self.as_slice().contains(&name)
    }
}

/// What to do with Node.js built-in modules
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeBuiltins {
    /// Externalize (for Node.js targets)
    External,
    /// Error if imported (for browser/edge)
    Error,
    /// Polyfill (for legacy browser support)
    Polyfill,
}

/// Trait for deployment targets that configure build behavior
///
/// This trait is implemented by deployment adapters (e.g., Vercel, Cloudflare)
/// to configure module resolution, export conditions, and output generation.
pub trait DeploymentTarget: Send + Sync {
    /// Unique identifier (e.g., "vercel-node", "cloudflare-workers")
    fn name(&self) -> &'static str;

    /// Runtime environment for this target
    fn runtime(&self) -> RuntimeEnvironment;

    /// Export conditions for module resolution
    fn conditions(&self) -> ExportConditions;

    /// How to handle Node.js built-ins
    fn node_builtins(&self) -> NodeBuiltins;

    /// Packages to always externalize
    fn external_packages(&self) -> Vec<String> {
        vec![]
    }

    /// Generate platform-specific output files
    ///
    /// This method is called after a successful build to generate any
    /// deployment-specific configuration files or adjust the output structure.
    fn generate_output(
        &self,
        _build_result: &crate::BuildResult,
        _output_dir: &Path,
    ) -> crate::Result<()> {
        // Default implementation: no additional output generation
        Ok(())
    }
}

//! Auto-detection of deployment targets from project files.

use crate::target::DeploymentTarget;
use crate::targets::{BrowserTarget, CloudflareWorkersTarget, VercelNodeTarget};
use std::path::Path;

/// Detect deployment target from project files
///
/// Checks for common deployment configuration files:
/// - `vercel.json` or `.vercel/` → VercelNodeTarget
/// - `wrangler.toml` or `_routes.json` → CloudflareWorkersTarget
/// - Otherwise → BrowserTarget (default)
pub fn detect_target(project_root: &Path) -> Box<dyn DeploymentTarget> {
    // Check for Vercel
    if project_root.join("vercel.json").exists() || project_root.join(".vercel").exists() {
        return Box::new(VercelNodeTarget);
    }

    // Check for Cloudflare Workers
    if project_root.join("wrangler.toml").exists() || project_root.join("_routes.json").exists() {
        return Box::new(CloudflareWorkersTarget);
    }

    // Default to browser
    Box::new(BrowserTarget)
}

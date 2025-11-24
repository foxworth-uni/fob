//! Path alias handling for module resolution.
//!
//! This module handles resolution of path aliases (e.g., "@" → "./src").

use std::path::Path;

/// Resolve a path alias (e.g., "@/components" → "./src/components").
pub fn resolve_path_alias(
    specifier: &str,
    _from: &Path,
    path_aliases: &rustc_hash::FxHashMap<String, String>,
) -> Option<String> {
    for (alias, target) in path_aliases {
        if specifier.starts_with(alias) {
            let rest = &specifier[alias.len()..];
            // Remove leading slash if present
            let rest = rest.strip_prefix('/').unwrap_or(rest);

            // Build resolved path - ensure it starts with ./ for relative resolution
            let resolved = if target.starts_with('/') {
                // Absolute path - shouldn't happen but handle it
                format!("{target}/{rest}")
            } else if target.starts_with('.') {
                // Already relative
                if rest.is_empty() {
                    target.clone()
                } else {
                    format!("{target}/{rest}")
                }
            } else {
                // Make it relative
                if rest.is_empty() {
                    format!("./{target}")
                } else {
                    format!("./{target}/{rest}")
                }
            };

            return Some(resolved);
        }
    }
    None
}


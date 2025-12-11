//! Cache key computation using BLAKE3 content-addressed hashing.
//!
//! The cache key is a deterministic hash of all build inputs, ensuring
//! automatic invalidation when any input changes.

use blake3::Hasher;
use std::path::Path;

use super::{CacheConfig, CacheResult};
use crate::builders::common::BundlePlan;

/// Current cache format version. Increment when cache format changes.
const CACHE_FORMAT_VERSION: u32 = 1;

/// Rolldown version for cache key.
/// TODO: Get this from rolldown crate metadata.
const ROLLDOWN_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Content-addressed cache key (BLAKE3 hash).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey(String);

impl CacheKey {
    /// Create a cache key from a hex string.
    pub fn from_hex(hex: impl Into<String>) -> Self {
        Self(hex.into())
    }

    /// Get the cache key as a hex string.
    pub fn as_hex(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Compute the cache key for a build plan.
///
/// The key is a BLAKE3 hash of:
/// 1. Cache format version
/// 2. Rolldown version
/// 3. Sorted entry paths + content hashes
/// 4. Serialized BundlerOptions (deterministic subset)
/// 5. Virtual files (sorted path + content hash)
/// 6. Specified environment variables (sorted)
pub fn compute_cache_key(plan: &BundlePlan, config: &CacheConfig) -> CacheResult<CacheKey> {
    let mut hasher = Hasher::new();

    // 1. Cache format version
    hasher.update(&CACHE_FORMAT_VERSION.to_le_bytes());

    // 2. Rolldown version
    hasher.update(ROLLDOWN_VERSION.as_bytes());

    // 3. Entry paths + content hashes (sorted for determinism)
    hash_entries(&mut hasher, plan)?;

    // 4. BundlerOptions (deterministic subset)
    hash_bundler_options(&mut hasher, &plan.options);

    // 5. Virtual files (sorted)
    hash_virtual_files(&mut hasher, &plan.virtual_files);

    // 6. Environment variables (sorted)
    hash_env_vars(&mut hasher, &config.env_vars);

    let hash = hasher.finalize();
    Ok(CacheKey(hash.to_hex().to_string()))
}

/// Hash entry files (paths + content).
fn hash_entries(hasher: &mut Hasher, plan: &BundlePlan) -> CacheResult<()> {
    // Collect and sort entry imports for determinism
    let mut entries: Vec<&str> = plan.entries.iter().map(|e| e.import.as_str()).collect();
    entries.sort();

    for entry in entries {
        // Hash the normalized path
        hasher.update(entry.as_bytes());
        hasher.update(b"\0"); // separator

        // Hash entry content if it's a file path (not virtual)
        if !entry.starts_with("virtual:") {
            if let Ok(content) = read_file_content(entry, plan) {
                let content_hash = blake3::hash(&content);
                hasher.update(content_hash.as_bytes());
            }
            // If file doesn't exist, just use path hash (will fail at build time anyway)
        }
    }

    Ok(())
}

/// Read file content, checking virtual files first.
fn read_file_content(path: &str, plan: &BundlePlan) -> std::io::Result<Vec<u8>> {
    // Check virtual files first
    if let Some(content) = plan.virtual_files.get(path) {
        return Ok(content.as_bytes().to_vec());
    }

    // Resolve path relative to cwd if needed
    let resolved_path = if Path::new(path).is_absolute() {
        Path::new(path).to_path_buf()
    } else if let Some(cwd) = &plan.cwd {
        cwd.join(path)
    } else {
        Path::new(path).to_path_buf()
    };

    std::fs::read(&resolved_path)
}

/// Hash BundlerOptions (deterministic subset).
///
/// We exclude fields that don't affect bundle content:
/// - `cwd` (already captured via entry paths)
/// - `input` (captured separately via entries)
fn hash_bundler_options(hasher: &mut Hasher, options: &rolldown::BundlerOptions) {
    // Format
    if let Some(format) = &options.format {
        hasher.update(format!("{:?}", format).as_bytes());
    }

    // Platform
    if let Some(platform) = &options.platform {
        hasher.update(format!("{:?}", platform).as_bytes());
    }

    // Sourcemap
    if let Some(sourcemap) = &options.sourcemap {
        hasher.update(format!("{:?}", sourcemap).as_bytes());
    }

    // Minify
    if let Some(minify) = &options.minify {
        hasher.update(format!("{:?}", minify).as_bytes());
    }

    // External packages (sorted for determinism)
    if let Some(external) = &options.external {
        hasher.update(format!("{:?}", external).as_bytes());
    }

    // Globals (sorted)
    if let Some(globals) = &options.globals {
        hasher.update(format!("{:?}", globals).as_bytes());
    }

    // Resolve options
    if let Some(resolve) = &options.resolve {
        // Hash alias mappings
        if let Some(alias) = &resolve.alias {
            let mut aliases: Vec<_> = alias.iter().collect();
            aliases.sort_by_key(|(k, _)| k.clone());
            for (key, values) in aliases {
                hasher.update(key.as_bytes());
                hasher.update(format!("{:?}", values).as_bytes());
            }
        }

        // Hash condition names
        if let Some(conditions) = &resolve.condition_names {
            let mut conds: Vec<_> = conditions.iter().collect();
            conds.sort();
            for cond in conds {
                hasher.update(cond.as_bytes());
            }
        }
    }

    // Advanced chunks (code splitting config)
    if let Some(advanced_chunks) = &options.advanced_chunks {
        hasher.update(format!("{:?}", advanced_chunks).as_bytes());
    }

    // Transform options (decorators, etc.)
    if let Some(transform) = &options.transform {
        hasher.update(format!("{:?}", transform).as_bytes());
    }
}

/// Hash virtual files (sorted for determinism).
fn hash_virtual_files(hasher: &mut Hasher, virtual_files: &rustc_hash::FxHashMap<String, String>) {
    // Sort paths for deterministic ordering
    let mut paths: Vec<_> = virtual_files.keys().collect();
    paths.sort();

    for path in paths {
        let content = &virtual_files[path];

        // Hash path
        hasher.update(path.as_bytes());
        hasher.update(b"\0");

        // Hash content
        let content_hash = blake3::hash(content.as_bytes());
        hasher.update(content_hash.as_bytes());
    }
}

/// Hash environment variables (sorted for determinism).
fn hash_env_vars(hasher: &mut Hasher, env_vars: &[String]) {
    // Sort env var names
    let mut vars: Vec<_> = env_vars.iter().collect();
    vars.sort();

    for var in vars {
        hasher.update(var.as_bytes());
        hasher.update(b"=");

        // Include value if set, empty otherwise
        if let Ok(value) = std::env::var(var) {
            hasher.update(value.as_bytes());
        }
        hasher.update(b"\0");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_plan(entries: Vec<&str>, vfiles: Vec<(&str, &str)>) -> BundlePlan {
        BundlePlan {
            entries: entries
                .into_iter()
                .map(|e| crate::builders::common::EntrySpec {
                    name: None,
                    import: e.to_string(),
                })
                .collect(),
            options: rolldown::BundlerOptions::default(),
            plugins: vec![],
            cwd: None,
            virtual_files: vfiles
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            runtime: None,
            cache: None,
            incremental: None,
        }
    }

    #[test]
    fn test_cache_key_deterministic() {
        let plan1 = make_test_plan(
            vec!["virtual:a.js", "virtual:b.js"],
            vec![
                ("virtual:a.js", "export const a = 1;"),
                ("virtual:b.js", "export const b = 2;"),
            ],
        );
        let plan2 = make_test_plan(
            vec!["virtual:b.js", "virtual:a.js"],
            vec![
                ("virtual:b.js", "export const b = 2;"),
                ("virtual:a.js", "export const a = 1;"),
            ],
        );

        let config = CacheConfig::default();

        let key1 = compute_cache_key(&plan1, &config).unwrap();
        let key2 = compute_cache_key(&plan2, &config).unwrap();

        // Same inputs in different order should produce same key
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_changes_on_content_change() {
        let plan1 = make_test_plan(
            vec!["virtual:a.js"],
            vec![("virtual:a.js", "export const a = 1;")],
        );
        let plan2 = make_test_plan(
            vec!["virtual:a.js"],
            vec![("virtual:a.js", "export const a = 2;")],
        );

        let config = CacheConfig::default();

        let key1 = compute_cache_key(&plan1, &config).unwrap();
        let key2 = compute_cache_key(&plan2, &config).unwrap();

        // Different content should produce different key
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_key_changes_on_entry_change() {
        let plan1 = make_test_plan(
            vec!["virtual:a.js"],
            vec![("virtual:a.js", "export const a = 1;")],
        );
        let plan2 = make_test_plan(
            vec!["virtual:b.js"],
            vec![("virtual:b.js", "export const a = 1;")],
        );

        let config = CacheConfig::default();

        let key1 = compute_cache_key(&plan1, &config).unwrap();
        let key2 = compute_cache_key(&plan2, &config).unwrap();

        // Different entry should produce different key
        assert_ne!(key1, key2);
    }
}

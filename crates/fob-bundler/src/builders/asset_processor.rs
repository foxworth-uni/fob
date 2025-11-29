//! Asset processor for production builds.
//!
//! Copies assets to the output directory with content-based hashing and
//! rewrites URLs in the bundled code to point to the hashed assets.

use super::asset_registry::{AssetInfo, AssetRegistry};
use crate::{Error, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// Process assets for production build.
///
/// # Steps
///
/// 1. Copy assets to output directory with content hashes
/// 2. Update registry with hashed filenames
/// 3. Return mapping of original paths to public URLs
///
/// # Arguments
///
/// * `registry` - Asset registry with discovered assets
/// * `out_dir` - Output directory for the build
/// * `assets_dir` - Subdirectory for assets (e.g., "assets")
/// * `public_path` - Base URL path for assets (e.g., "/")
///
/// # Returns
///
/// Map of original specifier → public URL for URL rewriting
pub fn process_assets(
    registry: &AssetRegistry,
    out_dir: &Path,
    assets_dir: &str,
    public_path: &str,
) -> Result<HashMap<String, String>> {
    let assets_output = out_dir.join(assets_dir);

    // Create assets directory
    std::fs::create_dir_all(&assets_output).map_err(|e| Error::IoError {
        message: format!(
            "Failed to create assets directory: {}",
            assets_output.display()
        ),
        source: e,
    })?;

    let mut url_map = HashMap::new();

    // Process each asset
    for asset in registry.all_assets() {
        let processed = process_single_asset(&asset, &assets_output)?;

        // Build public URL
        let public_url = format!(
            "{}{}{}/{}",
            public_path.trim_end_matches('/'),
            if public_path.ends_with('/') { "" } else { "/" },
            assets_dir,
            processed.filename
        );

        // Update registry with hash
        registry.set_content_hash(&asset.source_path, processed.hash.clone());

        // Map original specifier to public URL
        url_map.insert(asset.specifier.clone(), public_url);
    }

    Ok(url_map)
}

/// Information about a processed asset.
struct ProcessedAsset {
    /// Content hash
    hash: String,

    /// Final filename with hash
    filename: String,
}

/// Process a single asset file.
///
/// Reads the file, computes hash, copies to output with hashed name.
fn process_single_asset(asset: &AssetInfo, output_dir: &Path) -> Result<ProcessedAsset> {
    // Read asset content
    let content = std::fs::read(&asset.source_path).map_err(|e| Error::IoError {
        message: format!("Failed to read asset: {}", asset.source_path.display()),
        source: e,
    })?;

    // Compute content hash
    let hash = hash_content(&content);

    // Generate filename: [name]-[hash8].[ext]
    let filename = generate_filename(&asset.source_path, &hash)?;

    // Write to output directory
    let output_path = output_dir.join(&filename);
    std::fs::write(&output_path, &content).map_err(|e| Error::IoError {
        message: format!("Failed to write asset: {}", output_path.display()),
        source: e,
    })?;

    Ok(ProcessedAsset { hash, filename })
}

/// Hash asset content using SHA-256.
///
/// Returns hex-encoded hash (64 characters).
fn hash_content(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Generate asset filename with content hash.
///
/// Format: `[stem]-[hash8].[ext]`
///
/// Example: `file.wasm` with hash `abcd1234...` → `file-abcd1234.wasm`
fn generate_filename(path: &Path, hash: &str) -> Result<String> {
    let stem = path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
        Error::InvalidConfig(format!("Invalid asset filename: {}", path.display()))
    })?;

    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    // Use first 8 characters of hash
    let hash_short = &hash[..8.min(hash.len())];

    if ext.is_empty() {
        Ok(format!("{}-{}", stem, hash_short))
    } else {
        Ok(format!("{}-{}.{}", stem, hash_short, ext))
    }
}

/// Report of URL rewrite operations.
#[derive(Debug, Clone)]
pub struct RewriteReport {
    /// Number of URLs successfully rewritten
    pub replacements: usize,
    /// Specifiers that were found and rewritten
    pub rewritten_specifiers: Vec<String>,
    /// Specifiers from url_map that were not found in the code
    pub unused_specifiers: Vec<String>,
}

impl RewriteReport {
    fn new() -> Self {
        Self {
            replacements: 0,
            rewritten_specifiers: Vec::new(),
            unused_specifiers: Vec::new(),
        }
    }
}

/// Rewrite URLs in bundled JavaScript code using regex-based pattern matching.
///
/// Replaces `new URL(specifier, import.meta.url)` (optionally with `.href`)
/// with the mapped public URL from the url_map.
///
/// This function uses regex patterns that handle whitespace variations,
/// making it robust against minification and formatting differences.
///
/// # Arguments
///
/// * `code` - JavaScript code to rewrite
/// * `url_map` - Map of original specifier → public URL
///
/// # Returns
///
/// Result containing rewritten code and a report of operations
pub fn rewrite_urls_ast(
    code: &str,
    url_map: &HashMap<String, String>,
) -> Result<(String, RewriteReport)> {
    use regex::Regex;

    // If no mappings, return original code
    if url_map.is_empty() {
        return Ok((
            code.to_string(),
            RewriteReport {
                replacements: 0,
                rewritten_specifiers: Vec::new(),
                unused_specifiers: Vec::new(),
            },
        ));
    }

    let mut rewritten = code.to_string();
    let mut report = RewriteReport::new();

    // Escape special regex characters in specifiers
    for (specifier, public_url) in url_map {
        let escaped_specifier = regex::escape(specifier);

        // Pattern 1: new URL('specifier', import.meta.url) - single quotes, no .href
        let pattern1 = format!(
            r#"new\s+URL\s*\(\s*'({})'\s*,\s*import\.meta\.url\s*\)"#,
            escaped_specifier
        );
        if let Ok(re) = Regex::new(&pattern1) {
            if re.is_match(&rewritten) {
                rewritten = re
                    .replace_all(&rewritten, format!("'{}'", public_url))
                    .to_string();
                report.replacements += 1;
                report.rewritten_specifiers.push(specifier.clone());
            }
        }

        // Pattern 2: new URL("specifier", import.meta.url) - double quotes, no .href
        let pattern2 = format!(
            r#"new\s+URL\s*\(\s*"({})"\s*,\s*import\.meta\.url\s*\)"#,
            escaped_specifier
        );
        if let Ok(re) = Regex::new(&pattern2) {
            if re.is_match(&rewritten) {
                rewritten = re
                    .replace_all(&rewritten, format!("\"{}\"", public_url))
                    .to_string();
                report.replacements += 1;
                if !report.rewritten_specifiers.contains(specifier) {
                    report.rewritten_specifiers.push(specifier.clone());
                }
            }
        }

        // Pattern 3: new URL(`specifier`, import.meta.url) - template literals, no .href
        let pattern3 = format!(
            r#"new\s+URL\s*\(\s*`({})`\s*,\s*import\.meta\.url\s*\)"#,
            escaped_specifier
        );
        if let Ok(re) = Regex::new(&pattern3) {
            if re.is_match(&rewritten) {
                rewritten = re
                    .replace_all(&rewritten, format!("`{}`", public_url))
                    .to_string();
                report.replacements += 1;
                if !report.rewritten_specifiers.contains(specifier) {
                    report.rewritten_specifiers.push(specifier.clone());
                }
            }
        }

        // Pattern 4: new URL('specifier', import.meta.url).href - single quotes with .href
        let pattern4 = format!(
            r#"new\s+URL\s*\(\s*'({})'\s*,\s*import\.meta\.url\s*\)\s*\.href"#,
            escaped_specifier
        );
        if let Ok(re) = Regex::new(&pattern4) {
            if re.is_match(&rewritten) {
                rewritten = re
                    .replace_all(&rewritten, format!("'{}'", public_url))
                    .to_string();
                report.replacements += 1;
                if !report.rewritten_specifiers.contains(specifier) {
                    report.rewritten_specifiers.push(specifier.clone());
                }
            }
        }

        // Pattern 5: new URL("specifier", import.meta.url).href - double quotes with .href
        let pattern5 = format!(
            r#"new\s+URL\s*\(\s*"({})"\s*,\s*import\.meta\.url\s*\)\s*\.href"#,
            escaped_specifier
        );
        if let Ok(re) = Regex::new(&pattern5) {
            if re.is_match(&rewritten) {
                rewritten = re
                    .replace_all(&rewritten, format!("\"{}\"", public_url))
                    .to_string();
                report.replacements += 1;
                if !report.rewritten_specifiers.contains(specifier) {
                    report.rewritten_specifiers.push(specifier.clone());
                }
            }
        }

        // Pattern 6: new URL(`specifier`, import.meta.url).href - template literals with .href
        let pattern6 = format!(
            r#"new\s+URL\s*\(\s*`({})`\s*,\s*import\.meta\.url\s*\)\s*\.href"#,
            escaped_specifier
        );
        if let Ok(re) = Regex::new(&pattern6) {
            if re.is_match(&rewritten) {
                rewritten = re
                    .replace_all(&rewritten, format!("`{}`", public_url))
                    .to_string();
                report.replacements += 1;
                if !report.rewritten_specifiers.contains(specifier) {
                    report.rewritten_specifiers.push(specifier.clone());
                }
            }
        }
    }

    // Track unused specifiers
    for specifier in url_map.keys() {
        if !report.rewritten_specifiers.contains(specifier) {
            report.unused_specifiers.push(specifier.clone());
        }
    }

    Ok((rewritten, report))
}

/// Rewrite URLs in bundled JavaScript code (legacy string-based implementation).
///
/// This is kept for backward compatibility but should be replaced with
/// `rewrite_urls_ast` for better reliability.
///
/// # Arguments
///
/// * `code` - JavaScript code to rewrite
/// * `url_map` - Map of original specifier → public URL
///
/// # Returns
///
/// Rewritten code with updated URLs
pub fn rewrite_urls(code: &str, url_map: &HashMap<String, String>) -> String {
    // Use AST-based rewrite by default
    match rewrite_urls_ast(code, url_map) {
        Ok((rewritten, _report)) => rewritten,
        Err(_) => {
            // Fallback to old implementation if AST rewrite fails
            let mut rewritten = code.to_string();

            for (specifier, public_url) in url_map {
                // Pattern 1: new URL('specifier', import.meta.url)
                let pattern1 = format!("new URL('{}', import.meta.url)", specifier);
                let replacement1 = format!("'{}'", public_url);
                rewritten = rewritten.replace(&pattern1, &replacement1);

                // Pattern 2: new URL("specifier", import.meta.url)
                let pattern2 = format!("new URL(\"{}\", import.meta.url)", specifier);
                let replacement2 = format!("\"{}\"", public_url);
                rewritten = rewritten.replace(&pattern2, &replacement2);

                // Pattern 3: new URL(`specifier`, import.meta.url)
                let pattern3 = format!("new URL(`{}`, import.meta.url)", specifier);
                let replacement3 = format!("`{}`", public_url);
                rewritten = rewritten.replace(&pattern3, &replacement3);

                // Pattern 4: new URL('specifier', import.meta.url).href (used in dynamic imports)
                let pattern4 = format!("new URL('{}', import.meta.url).href", specifier);
                let replacement4 = format!("'{}'", public_url);
                rewritten = rewritten.replace(&pattern4, &replacement4);

                // Pattern 5: new URL("specifier", import.meta.url).href
                let pattern5 = format!("new URL(\"{}\", import.meta.url).href", specifier);
                let replacement5 = format!("\"{}\"", public_url);
                rewritten = rewritten.replace(&pattern5, &replacement5);

                // Pattern 6: new URL(`specifier`, import.meta.url).href
                let pattern6 = format!("new URL(`{}`, import.meta.url).href", specifier);
                let replacement6 = format!("`{}`", public_url);
                rewritten = rewritten.replace(&pattern6, &replacement6);
            }

            rewritten
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_content() {
        let content = b"Hello, World!";
        let hash = hash_content(content);

        // SHA-256 produces 64 hex characters
        assert_eq!(hash.len(), 64);

        // Same content produces same hash
        let hash2 = hash_content(content);
        assert_eq!(hash, hash2);

        // Different content produces different hash
        let hash3 = hash_content(b"Different");
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_generate_filename() {
        let path = Path::new("file.wasm");
        let hash = "abcd1234567890abcdef";

        let filename = generate_filename(path, hash).unwrap();
        assert_eq!(filename, "file-abcd1234.wasm");
    }

    #[test]
    fn test_generate_filename_no_extension() {
        let path = Path::new("LICENSE");
        let hash = "abcd1234567890abcdef";

        let filename = generate_filename(path, hash).unwrap();
        assert_eq!(filename, "LICENSE-abcd1234");
    }

    #[test]
    fn test_rewrite_urls() {
        let code = r#"
            const url1 = new URL('./file.wasm', import.meta.url);
            const url2 = new URL("./image.png", import.meta.url);
            const url3 = new URL(`./font.woff2`, import.meta.url);
        "#;

        let mut url_map = HashMap::new();
        url_map.insert(
            "./file.wasm".to_string(),
            "/assets/file-abc123.wasm".to_string(),
        );
        url_map.insert(
            "./image.png".to_string(),
            "/assets/image-def456.png".to_string(),
        );
        url_map.insert(
            "./font.woff2".to_string(),
            "/assets/font-ghi789.woff2".to_string(),
        );

        let rewritten = rewrite_urls(code, &url_map);

        assert!(rewritten.contains("'/assets/file-abc123.wasm'"));
        assert!(rewritten.contains("\"/assets/image-def456.png\""));
        assert!(rewritten.contains("`/assets/font-ghi789.woff2`"));

        // Original patterns should be gone
        assert!(!rewritten.contains("new URL('./file.wasm'"));
        assert!(!rewritten.contains("new URL(\"./image.png\""));
        assert!(!rewritten.contains("new URL(`./font.woff2`"));
    }

    #[test]
    fn test_rewrite_urls_with_href() {
        // Test .href patterns used in dynamic imports
        let code = r#"
            const module = await import(new URL('../wasm/web/joy_bundler_wasm.js', import.meta.url).href);
            const url1 = new URL('./file.wasm', import.meta.url).href;
            const url2 = new URL("./image.png", import.meta.url).href;
            const url3 = new URL(`./font.woff2`, import.meta.url).href;
        "#;

        let mut url_map = HashMap::new();
        url_map.insert(
            "../wasm/web/joy_bundler_wasm.js".to_string(),
            "/__fob_assets__/joy_bundler_wasm.js".to_string(),
        );
        url_map.insert(
            "./file.wasm".to_string(),
            "/__fob_assets__/file.wasm".to_string(),
        );
        url_map.insert(
            "./image.png".to_string(),
            "/__fob_assets__/image.png".to_string(),
        );
        url_map.insert(
            "./font.woff2".to_string(),
            "/__fob_assets__/font.woff2".to_string(),
        );

        let rewritten = rewrite_urls(code, &url_map);

        // Check that .href patterns are rewritten
        assert!(rewritten.contains("'/__fob_assets__/joy_bundler_wasm.js'"));
        assert!(rewritten.contains("'/__fob_assets__/file.wasm'"));
        assert!(rewritten.contains("\"/__fob_assets__/image.png\""));
        assert!(rewritten.contains("`/__fob_assets__/font.woff2`"));

        // Original patterns should be gone
        assert!(!rewritten.contains("new URL('../wasm/web/joy_bundler_wasm.js'"));
        assert!(!rewritten.contains("new URL('./file.wasm', import.meta.url).href"));
        assert!(!rewritten.contains("new URL(\"./image.png\", import.meta.url).href"));
        assert!(!rewritten.contains("new URL(`./font.woff2`, import.meta.url).href"));
    }

    #[test]
    fn test_rewrite_urls_ast_minified() {
        // Test minified code (no spaces)
        let code = r#"const url=new URL('../wasm/web/joy_bundler_wasm.js',import.meta.url).href;"#;

        let mut url_map = HashMap::new();
        url_map.insert(
            "../wasm/web/joy_bundler_wasm.js".to_string(),
            "/__fob_assets__/joy_bundler_wasm.js".to_string(),
        );

        let (rewritten, report) = rewrite_urls_ast(code, &url_map).unwrap();

        assert_eq!(report.replacements, 1);
        assert!(rewritten.contains("'/__fob_assets__/joy_bundler_wasm.js'"));
        assert!(!rewritten.contains("new URL"));
    }

    #[test]
    fn test_rewrite_urls_ast_spaced() {
        // Test code with extra spaces
        let code =
            r#"const url = new URL ( '../wasm/web/joy_bundler_wasm.js' , import.meta.url ) .href;"#;

        let mut url_map = HashMap::new();
        url_map.insert(
            "../wasm/web/joy_bundler_wasm.js".to_string(),
            "/__fob_assets__/joy_bundler_wasm.js".to_string(),
        );

        let (rewritten, report) = rewrite_urls_ast(code, &url_map).unwrap();

        assert_eq!(report.replacements, 1);
        assert!(rewritten.contains("'/__fob_assets__/joy_bundler_wasm.js'"));
    }

    #[test]
    fn test_rewrite_urls_ast_template_literal() {
        // Test template literal
        let code = r#"const url = new URL(`../wasm/web/joy_bundler_wasm.js`, import.meta.url);"#;

        let mut url_map = HashMap::new();
        url_map.insert(
            "../wasm/web/joy_bundler_wasm.js".to_string(),
            "/__fob_assets__/joy_bundler_wasm.js".to_string(),
        );

        let (rewritten, report) = rewrite_urls_ast(code, &url_map).unwrap();

        assert_eq!(report.replacements, 1);
        assert!(rewritten.contains("`/__fob_assets__/joy_bundler_wasm.js`"));
    }

    #[test]
    fn test_rewrite_urls_ast_preserves_quote_type() {
        // Test that quote types are preserved
        let code = r#"
            const url1 = new URL('./file.wasm', import.meta.url);
            const url2 = new URL("./image.png", import.meta.url);
            const url3 = new URL(`./font.woff2`, import.meta.url);
        "#;

        let mut url_map = HashMap::new();
        url_map.insert("./file.wasm".to_string(), "/assets/file.wasm".to_string());
        url_map.insert("./image.png".to_string(), "/assets/image.png".to_string());
        url_map.insert("./font.woff2".to_string(), "/assets/font.woff2".to_string());

        let (rewritten, report) = rewrite_urls_ast(code, &url_map).unwrap();

        assert_eq!(report.replacements, 3);
        assert!(rewritten.contains("'/assets/file.wasm'"));
        assert!(rewritten.contains("\"/assets/image.png\""));
        assert!(rewritten.contains("`/assets/font.woff2`"));
    }

    #[test]
    fn test_rewrite_urls_ast_unused_specifiers() {
        // Test that unused specifiers are tracked
        let code = r#"const url = new URL('./file.wasm', import.meta.url);"#;

        let mut url_map = HashMap::new();
        url_map.insert("./file.wasm".to_string(), "/assets/file.wasm".to_string());
        url_map.insert("./unused.png".to_string(), "/assets/unused.png".to_string());

        let (_rewritten, report) = rewrite_urls_ast(code, &url_map).unwrap();

        assert_eq!(report.replacements, 1);
        assert_eq!(report.rewritten_specifiers.len(), 1);
        assert_eq!(report.unused_specifiers.len(), 1);
        assert!(
            report
                .unused_specifiers
                .contains(&"./unused.png".to_string())
        );
    }

    #[test]
    fn test_rewrite_urls_ast_empty_map() {
        // Test with empty url_map
        let code = r#"const url = new URL('./file.wasm', import.meta.url);"#;

        let url_map = HashMap::new();

        let (rewritten, report) = rewrite_urls_ast(code, &url_map).unwrap();

        assert_eq!(report.replacements, 0);
        assert_eq!(rewritten, code);
    }

    #[test]
    fn test_process_single_asset() {
        use std::fs;
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let source_file = temp.path().join("test.wasm");
        let output_dir = temp.path().join("out");

        // Create source asset
        fs::write(&source_file, b"test content").unwrap();
        fs::create_dir(&output_dir).unwrap();

        // Create asset info
        let asset = AssetInfo {
            source_path: source_file.clone(),
            referrer: "index.js".to_string(),
            specifier: "./test.wasm".to_string(),
            content_type: "application/wasm".to_string(),
            size: Some(12),
            url_path: None,
            content_hash: None,
        };

        // Process asset
        let processed = process_single_asset(&asset, &output_dir).unwrap();

        assert_eq!(processed.hash.len(), 64);
        assert!(processed.filename.starts_with("test-"));
        assert!(processed.filename.ends_with(".wasm"));

        // Check output file exists
        let output_file = output_dir.join(&processed.filename);
        assert!(output_file.exists());

        // Verify content and size
        let content = fs::read(&output_file).unwrap();
        assert_eq!(content, b"test content");
        assert_eq!(content.len(), 12);
    }
}

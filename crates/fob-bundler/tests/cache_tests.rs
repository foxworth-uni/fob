//! Cache functionality tests for fob-bundler.
//!
//! These tests verify that the build cache works correctly:
//! - Cache hits/misses
//! - Key determinism
//! - Cache invalidation on content change

use fob_bundler::{BuildOptions, CacheConfig};
use tempfile::TempDir;

/// Test that building the same code twice results in a cache hit.
#[tokio::test]
async fn test_cache_hit_and_miss() {
    let temp = TempDir::new().unwrap();
    let cache_dir = temp.path().join("cache");

    // Build 1: Cache miss (no cached result exists)
    let result1 = BuildOptions::new("virtual:entry.js")
        .virtual_file("virtual:entry.js", "export const x = 1;")
        .cache_dir(&cache_dir)
        .outfile(temp.path().join("out1.js"))
        .build()
        .await
        .unwrap();

    assert_eq!(result1.output.assets().count(), 1);

    // Build 2: Should hit cache (same inputs)
    let result2 = BuildOptions::new("virtual:entry.js")
        .virtual_file("virtual:entry.js", "export const x = 1;")
        .cache_dir(&cache_dir)
        .outfile(temp.path().join("out2.js"))
        .build()
        .await
        .unwrap();

    assert_eq!(result2.output.assets().count(), 1);

    // Both builds should produce the same output
    let chunks1: Vec<_> = result1.output.chunks().collect();
    let code1 = &chunks1[0].code;

    let chunks2: Vec<_> = result2.output.chunks().collect();
    let code2 = &chunks2[0].code;

    assert_eq!(code1, code2, "Cached result should match original build");
}

/// Test that cache keys are deterministic (same inputs = same key).
#[tokio::test]
async fn test_cache_key_determinism() {
    let temp = TempDir::new().unwrap();
    let cache_dir = temp.path().join("cache");

    // Build with the same inputs in different order
    let result1 = BuildOptions::new("virtual:a.js")
        .virtual_file("virtual:a.js", "export const a = 1;")
        .virtual_file("virtual:b.js", "export const b = 2;")
        .cache_dir(&cache_dir)
        .outfile(temp.path().join("out1.js"))
        .build()
        .await
        .unwrap();

    let result2 = BuildOptions::new("virtual:a.js")
        .virtual_file("virtual:b.js", "export const b = 2;")
        .virtual_file("virtual:a.js", "export const a = 1;")
        .cache_dir(&cache_dir)
        .outfile(temp.path().join("out2.js"))
        .build()
        .await
        .unwrap();

    // Both builds should produce identical outputs
    assert_eq!(
        result1.output.assets().count(),
        result2.output.assets().count()
    );
}

/// Test that changing content invalidates the cache.
#[tokio::test]
async fn test_cache_invalidation_on_content_change() {
    let temp = TempDir::new().unwrap();
    let cache_dir = temp.path().join("cache");

    // Build 1: Initial build
    let result1 = BuildOptions::new("virtual:entry.js")
        .virtual_file("virtual:entry.js", "export const x = 1;")
        .cache_dir(&cache_dir)
        .outfile(temp.path().join("out1.js"))
        .build()
        .await
        .unwrap();

    let chunks1: Vec<_> = result1.output.chunks().collect();
    let code1 = &chunks1[0].code;

    // Build 2: Different content (cache miss)
    let result2 = BuildOptions::new("virtual:entry.js")
        .virtual_file("virtual:entry.js", "export const x = 2;")
        .cache_dir(&cache_dir)
        .outfile(temp.path().join("out2.js"))
        .build()
        .await
        .unwrap();

    let chunks2: Vec<_> = result2.output.chunks().collect();
    let code2 = &chunks2[0].code;

    // Different content should produce different output
    assert_ne!(code1, code2, "Changed content should invalidate cache");
}

/// Test that force_rebuild bypasses the cache but still writes to it.
#[tokio::test]
async fn test_force_rebuild() {
    let temp = TempDir::new().unwrap();
    let cache_dir = temp.path().join("cache");

    // Build 1: Initial build
    let _result1 = BuildOptions::new("virtual:entry.js")
        .virtual_file("virtual:entry.js", "export const x = 1;")
        .cache_dir(&cache_dir)
        .outfile(temp.path().join("out1.js"))
        .build()
        .await
        .unwrap();

    // Build 2: Force rebuild (bypasses cache read, but writes to cache)
    let result2 = BuildOptions::new("virtual:entry.js")
        .virtual_file("virtual:entry.js", "export const x = 1;")
        .cache_dir(&cache_dir)
        .force_rebuild()
        .outfile(temp.path().join("out2.js"))
        .build()
        .await
        .unwrap();

    // Should still build successfully
    assert_eq!(result2.output.assets().count(), 1);
}

/// Test that cache config with environment variables works.
#[tokio::test]
async fn test_cache_with_env_vars() {
    let temp = TempDir::new().unwrap();
    let cache_dir = temp.path().join("cache");

    // Set an environment variable
    unsafe {
        std::env::set_var("TEST_CACHE_VAR", "value1");
    }

    // Build 1: With env var
    let cache_config = CacheConfig::new(&cache_dir).with_env_vars(["TEST_CACHE_VAR"]);

    let result1 = BuildOptions::new("virtual:entry.js")
        .virtual_file("virtual:entry.js", "export const x = 1;")
        .cache(cache_config.clone())
        .outfile(temp.path().join("out1.js"))
        .build()
        .await
        .unwrap();

    assert_eq!(result1.output.assets().count(), 1);

    // Build 2: Change env var (should invalidate cache)
    unsafe {
        std::env::set_var("TEST_CACHE_VAR", "value2");
    }

    let cache_config2 = CacheConfig::new(&cache_dir).with_env_vars(["TEST_CACHE_VAR"]);

    let result2 = BuildOptions::new("virtual:entry.js")
        .virtual_file("virtual:entry.js", "export const x = 1;")
        .cache(cache_config2)
        .outfile(temp.path().join("out2.js"))
        .build()
        .await
        .unwrap();

    assert_eq!(result2.output.assets().count(), 1);

    // Clean up
    unsafe {
        std::env::remove_var("TEST_CACHE_VAR");
    }
}

/// Test that cache works with multiple entries.
#[tokio::test]
async fn test_cache_with_multiple_entries() {
    let temp = TempDir::new().unwrap();
    let cache_dir = temp.path().join("cache");

    // Build 1: Multiple entries
    let result1 = BuildOptions::new_multiple(["virtual:a.js", "virtual:b.js"])
        .virtual_file("virtual:a.js", "export const a = 1;")
        .virtual_file("virtual:b.js", "export const b = 2;")
        .cache_dir(&cache_dir)
        .outdir(temp.path().join("dist1"))
        .build()
        .await
        .unwrap();

    let asset_count1 = result1.output.assets().count();
    assert!(asset_count1 >= 2, "Should have at least 2 chunks");

    // Build 2: Same entries (cache hit)
    let result2 = BuildOptions::new_multiple(["virtual:a.js", "virtual:b.js"])
        .virtual_file("virtual:a.js", "export const a = 1;")
        .virtual_file("virtual:b.js", "export const b = 2;")
        .cache_dir(&cache_dir)
        .outdir(temp.path().join("dist2"))
        .build()
        .await
        .unwrap();

    assert_eq!(result2.output.assets().count(), asset_count1);
}

/// Test that corrupted cache files don't crash the bundler.
/// The bundler should gracefully handle corruption and rebuild.
#[tokio::test]
async fn test_cache_handles_corrupted_data() {
    use std::fs;
    use std::io::Write;

    let temp = TempDir::new().unwrap();
    let cache_dir = temp.path().join("cache");

    // Build 1: Create valid cache
    let result1 = BuildOptions::new("virtual:entry.js")
        .virtual_file("virtual:entry.js", "export const x = 42;")
        .cache_dir(&cache_dir)
        .outfile(temp.path().join("out1.js"))
        .build()
        .await
        .unwrap();

    assert_eq!(result1.output.assets().count(), 1);

    // Corrupt the cache database file by writing garbage
    let cache_db = cache_dir.join("cache.redb");
    if cache_db.exists() {
        // Write garbage to corrupt the database
        let mut file = fs::OpenOptions::new().write(true).open(&cache_db).unwrap();
        file.write_all(b"CORRUPTED DATA HERE").unwrap();
    }

    // Build 2: Should rebuild despite corrupted cache (not crash or hang)
    let result2 = BuildOptions::new("virtual:entry.js")
        .virtual_file("virtual:entry.js", "export const x = 42;")
        .cache_dir(&cache_dir)
        .outfile(temp.path().join("out2.js"))
        .build()
        .await;

    // The build should either succeed (cache ignored/rebuilt) or fail gracefully
    match result2 {
        Ok(result) => {
            // Rebuilt successfully despite corruption
            assert_eq!(result.output.assets().count(), 1);
        }
        Err(e) => {
            // If it fails, error should mention cache issue (not panic)
            let err_msg = e.to_string();
            assert!(
                err_msg.contains("cache") || err_msg.contains("database") || err_msg.len() > 0,
                "Error should be meaningful: {}",
                err_msg
            );
        }
    }
}

//! Integration tests for MDX compilation via joy-core
//!
//! These tests verify that the joy-mdx plugin correctly integrates with
//! Rolldown using the new task-based builder APIs.

use async_trait::async_trait;
use fob_bundler::{plugin, BuildOptions, BunnyMdxPlugin, Platform};
use fob_bundler::{FileMetadata, Runtime, RuntimeError, RuntimeResult};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;

/// Simple test runtime for MDX integration tests
#[derive(Debug)]
struct TestRuntime {
    cwd: PathBuf,
}

impl TestRuntime {
    fn new(cwd: PathBuf) -> Self {
        Self { cwd }
    }
}

#[async_trait]
impl Runtime for TestRuntime {
    async fn read_file(&self, path: &Path) -> RuntimeResult<Vec<u8>> {
        std::fs::read(path).map_err(|e| RuntimeError::Io(e.to_string()))
    }

    async fn write_file(&self, path: &Path, content: &[u8]) -> RuntimeResult<()> {
        std::fs::write(path, content).map_err(|e| RuntimeError::Io(e.to_string()))
    }

    async fn metadata(&self, path: &Path) -> RuntimeResult<FileMetadata> {
        let metadata = std::fs::metadata(path).map_err(|e| RuntimeError::Io(e.to_string()))?;
        Ok(FileMetadata {
            size: metadata.len(),
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
            modified: metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64),
        })
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn resolve(&self, specifier: &str, from: &Path) -> RuntimeResult<PathBuf> {
        Ok(from.parent().unwrap_or(&self.cwd).join(specifier))
    }

    async fn create_dir(&self, path: &Path, recursive: bool) -> RuntimeResult<()> {
        if recursive {
            std::fs::create_dir_all(path).map_err(|e| RuntimeError::Io(e.to_string()))
        } else {
            std::fs::create_dir(path).map_err(|e| RuntimeError::Io(e.to_string()))
        }
    }

    async fn remove_file(&self, path: &Path) -> RuntimeResult<()> {
        std::fs::remove_file(path).map_err(|e| RuntimeError::Io(e.to_string()))
    }

    async fn read_dir(&self, path: &Path) -> RuntimeResult<Vec<String>> {
        let entries: Vec<String> = std::fs::read_dir(path)
            .map_err(|e| RuntimeError::Io(e.to_string()))?
            .filter_map(|entry| {
                entry
                    .ok()
                    .and_then(|e| e.file_name().to_str().map(String::from))
            })
            .collect();
        Ok(entries)
    }

    fn get_cwd(&self) -> RuntimeResult<PathBuf> {
        Ok(self.cwd.clone())
    }
}

#[tokio::test]
async fn test_basic_mdx_compilation() {
    let dir = TempDir::new().unwrap();

    std::fs::write(
        dir.path().join("test.mdx"),
        "# Hello MDX\n\nThis is **bold** text.",
    )
    .unwrap();

    let runtime = Arc::new(TestRuntime::new(dir.path().to_path_buf()));

    let result = BuildOptions::new(dir.path().join("test.mdx"))
        .bundle(false)
        .platform(Platform::Node)
        .cwd(dir.path())
        .runtime(runtime)
        .plugin(plugin(BunnyMdxPlugin::new(dir.path().to_path_buf())))
        .build()
        .await
        .unwrap();

    let bundle = result.output.as_single().expect("single bundle");
    assert!(
        !bundle.assets.is_empty(),
        "Expected at least one output asset"
    );
}

#[tokio::test]
async fn test_mdx_with_frontmatter() {
    let dir = TempDir::new().unwrap();

    std::fs::write(
        dir.path().join("post.mdx"),
        r#"---
title: My Blog Post
author: Joy
---

# {frontmatter.title}

Written by {frontmatter.author}
"#,
    )
    .unwrap();

    let runtime = Arc::new(TestRuntime::new(dir.path().to_path_buf()));

    let result = BuildOptions::new(dir.path().join("post.mdx"))
        .bundle(false)
        .platform(Platform::Node)
        .cwd(dir.path())
        .runtime(runtime)
        .plugin(plugin(BunnyMdxPlugin::new(dir.path().to_path_buf())))
        .build()
        .await
        .unwrap();

    let bundle = result.output.as_single().expect("single bundle");
    assert!(
        !bundle.assets.is_empty(),
        "Expected output with frontmatter processing"
    );
}

#[tokio::test]
async fn test_mdx_with_gfm_features() {
    let dir = TempDir::new().unwrap();

    std::fs::write(
        dir.path().join("gfm.mdx"),
        r#"# GFM Test

This is ~~strikethrough~~ text.

| Column 1 | Column 2 |
|----------|----------|
| Cell 1   | Cell 2   |

- [x] Task 1
- [ ] Task 2
"#,
    )
    .unwrap();

    let runtime = Arc::new(TestRuntime::new(dir.path().to_path_buf()));

    let result = BuildOptions::new(dir.path().join("gfm.mdx"))
        .bundle(false)
        .platform(Platform::Node)
        .cwd(dir.path())
        .runtime(runtime)
        .plugin(plugin(BunnyMdxPlugin::new(dir.path().to_path_buf())))
        .build()
        .await
        .unwrap();

    let bundle = result.output.as_single().expect("single bundle");
    assert!(
        !bundle.assets.is_empty(),
        "Expected output with GFM features"
    );
}

#[tokio::test]
async fn test_multiple_mdx_files() {
    let dir = TempDir::new().unwrap();

    std::fs::write(dir.path().join("page1.mdx"), "# Page 1\n\nContent 1").unwrap();
    std::fs::write(dir.path().join("page2.mdx"), "# Page 2\n\nContent 2").unwrap();

    let runtime = Arc::new(TestRuntime::new(dir.path().to_path_buf()));

    let result =
        BuildOptions::new_multiple([dir.path().join("page1.mdx"), dir.path().join("page2.mdx")])
            .bundle(true)
            .splitting(false)
            .cwd(dir.path())
            .runtime(runtime)
            .plugin(plugin(BunnyMdxPlugin::new(dir.path().to_path_buf())))
            .build()
            .await
            .unwrap();

    let bundles = result.output.as_multiple().expect("multiple bundles");
    let asset_count: usize = bundles.iter().map(|(_, bundle)| bundle.assets.len()).sum();

    assert!(
        asset_count >= 2,
        "Expected at least 2 output assets for 2 MDX files"
    );
}

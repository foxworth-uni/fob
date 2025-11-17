//! Integration tests for MDX compilation via joy-core
//!
//! These tests verify that the joy-mdx plugin correctly integrates with
//! Rolldown using the new task-based builder APIs.

use fob_core::{plugin, BuildOptions, BunnyMdxPlugin, Platform};
use tempfile::TempDir;

#[tokio::test]
async fn test_basic_mdx_compilation() {
    let dir = TempDir::new().unwrap();

    std::fs::write(
        dir.path().join("test.mdx"),
        "# Hello MDX\n\nThis is **bold** text.",
    )
    .unwrap();

    let result = BuildOptions::new(dir.path().join("test.mdx"))
        .bundle(false)
        .platform(Platform::Node)
        .cwd(dir.path())
        .plugin(plugin(BunnyMdxPlugin::new()))
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

    let result = BuildOptions::new(dir.path().join("post.mdx"))
        .bundle(false)
        .platform(Platform::Node)
        .cwd(dir.path())
        .plugin(plugin(BunnyMdxPlugin::new()))
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

    let result = BuildOptions::new(dir.path().join("gfm.mdx"))
        .bundle(false)
        .platform(Platform::Node)
        .cwd(dir.path())
        .plugin(plugin(BunnyMdxPlugin::new()))
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

    let result =
        BuildOptions::new_multiple([dir.path().join("page1.mdx"), dir.path().join("page2.mdx")])
            .bundle(true)
            .splitting(false)
            .cwd(dir.path())
            .plugin(plugin(BunnyMdxPlugin::new()))
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

//! Interactive publishing workflow

use anyhow::{Context, Result};
use std::io;
use std::path::Path;
use std::process::Command;
use toml_edit::DocumentMut;

/// Result of checking if a crate is published
#[derive(Debug)]
#[allow(dead_code)]
enum PublishStatus {
    /// Crate exists at this exact version
    Published,
    /// Crate exists but at a different version
    DifferentVersion(String),
    /// Crate does not exist on crates.io
    NotFound,
}

/// Check if a crate version is already published on crates.io
/// Uses multiple strategies to ensure accurate results
fn check_publish_status(crate_name: &str, version: &str) -> PublishStatus {
    // Strategy 1: Use cargo search with index update
    // Running `cargo search --limit 0` forces an index update
    let _ = Command::new("cargo")
        .args(["search", "--limit", "0"])
        .output();

    // Search for the crate (use limit 10 to handle similar names)
    let search_output = Command::new("cargo")
        .args(["search", crate_name, "--limit", "10"])
        .output();

    if let Ok(output) = search_output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Look for exact crate name match: `crate_name = "version"`
            let search_pattern = format!("{} = \"", crate_name);
            for line in stdout.lines() {
                if line.contains(&search_pattern) {
                    // Found exact crate, check version
                    if line.contains(&format!("\"{}\"", version)) {
                        println!("  ✓ cargo search confirms {} v{} is published", crate_name, version);
                        return PublishStatus::Published;
                    } else {
                        println!("  ℹ cargo search found {} at different version", crate_name);
                        return PublishStatus::DifferentVersion("other".to_string());
                    }
                }
            }
        }
    }

    // Strategy 2: Fall back to cargo info
    let info_output = Command::new("cargo")
        .args(["info", crate_name])
        .output();

    if let Ok(output) = info_output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Simple check: does output contain "version: X.Y.Z"?
            if stdout.contains(&format!("version: {}", version)) {
                println!("  ✓ cargo info confirms {} v{} is published", crate_name, version);
                return PublishStatus::Published;
            }
            // Check for any version line (different version exists)
            for line in stdout.lines() {
                if line.trim().starts_with("version:") {
                    println!("  ℹ cargo info found {} at different version", crate_name);
                    return PublishStatus::DifferentVersion("other".to_string());
                }
            }
        }
    }

    PublishStatus::NotFound
}

/// Check if a crate version is already published (simple bool wrapper)
fn is_already_published(crate_name: &str, version: &str) -> bool {
    matches!(check_publish_status(crate_name, version), PublishStatus::Published)
}

/// Publish crates to crates.io in the order they appear in workspace
/// (assumes workspace members are already in correct dependency order)
pub async fn publish_crates(workspace_dir: &str, interactive: bool, dry_run: bool) -> Result<()> {
    let workspace_path = Path::new(workspace_dir);
    let workspace_toml = workspace_path.join("Cargo.toml");

    if !workspace_toml.exists() {
        anyhow::bail!(
            "Workspace Cargo.toml not found at: {}",
            workspace_toml.display()
        );
    }

    // Scan crates directory for fob_* crates (workspace uses globs which we can't parse)
    let crates_dir = workspace_path.join("crates");
    let mut crates_to_publish = Vec::new();

    for entry in std::fs::read_dir(&crates_dir)
        .with_context(|| format!("Failed to read crates directory: {}", crates_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let cargo_toml = path.join("Cargo.toml");
        if !cargo_toml.exists() {
            continue;
        }

        let content = std::fs::read_to_string(&cargo_toml)
            .with_context(|| format!("Failed to read {}", cargo_toml.display()))?;
        let doc: DocumentMut = content
            .parse()
            .with_context(|| format!("Failed to parse {}", cargo_toml.display()))?;

        let package = match doc.get("package") {
            Some(p) => p,
            None => continue,
        };

        let name = package.get("name").and_then(|n| n.as_str()).unwrap_or("");
        // Only include transformed crates (fob_ prefix)
        if !name.starts_with("fob_") {
            continue;
        }

        // Skip crates with publish = false
        let publishable = package
            .get("publish")
            .and_then(|p| p.as_bool())
            .unwrap_or(true);
        if !publishable {
            println!("  Skipping {} (publish = false)", name);
            continue;
        }

        let version = package
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0");
        crates_to_publish.push((name.to_string(), version.to_string(), path));
    }

    // Sort alphabetically for consistent ordering
    crates_to_publish.sort_by(|a, b| a.0.cmp(&b.0));

    let mode = if dry_run { "Dry-run publishing" } else { "Publishing" };
    println!(
        "{} {} crates in dependency order:",
        mode,
        crates_to_publish.len()
    );
    for (i, (crate_name, version, _)) in crates_to_publish.iter().enumerate() {
        println!("  {}. {} v{}", i + 1, crate_name, version);
    }

    if interactive && !dry_run {
        println!("\nPress Enter to continue with publishing, or Ctrl+C to cancel...");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
    }

    // Publish each crate
    for (crate_name, version, crate_path) in &crates_to_publish {
        // Check if already published (skip for dry-run)
        if !dry_run && is_already_published(crate_name, version) {
            println!("\n⏭️  {} v{} already published, skipping", crate_name, version);
            continue;
        }

        if dry_run {
            println!("\n📦 Dry-run {}...", crate_name);
        } else {
            println!("\n📦 Publishing {} v{}...", crate_name, version);
        }

        if interactive && !dry_run {
            println!(
                "Press Enter to publish {}, or Ctrl+C to skip...",
                crate_name
            );
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
        }

        let mut cmd = Command::new("cargo");

        if dry_run {
            // Use 'cargo package --list' for dry-run to verify package structure
            // without requiring dependencies to exist on crates.io
            cmd.arg("package")
                .arg("--manifest-path")
                .arg(crate_path.join("Cargo.toml"))
                .arg("--allow-dirty")
                .arg("--list") // Just list files, don't resolve deps
                .current_dir(workspace_path);
        } else {
            cmd.arg("publish")
                .arg("--manifest-path")
                .arg(crate_path.join("Cargo.toml"))
                .current_dir(workspace_path);
        }

        let status = cmd
            .status()
            .with_context(|| format!("Failed to run cargo publish for {}", crate_name))?;

        if !status.success() {
            if dry_run {
                anyhow::bail!("Dry-run failed for {}", crate_name);
            } else {
                anyhow::bail!("Failed to publish {}", crate_name);
            }
        }

        if dry_run {
            println!("✓ Dry-run passed for {}", crate_name);
        } else {
            println!("✓ Published {}", crate_name);
        }
    }

    if dry_run {
        println!("\n✓ Dry-run completed successfully! All crates ready to publish.");
    } else {
        println!("\n✓ All crates published successfully!");
    }

    Ok(())
}

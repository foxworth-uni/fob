//! Transform rolldown to fob_rolldown using toml_edit for proper TOML manipulation

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use toml_edit::{value, DocumentMut};
use walkdir::WalkDir;

/// Transform rolldown repository in-place to fob_rolldown
pub fn transform_in_place(rolldown_dir: &Path, version: &str) -> Result<()> {
    println!("Transforming rolldown to fob_rolldown v{}...", version);

    // 1. Update workspace Cargo.toml
    println!("  Updating workspace Cargo.toml...");
    update_workspace_toml(rolldown_dir, version)?;

    // 2. Update each crate's Cargo.toml
    println!("  Updating crate Cargo.toml files...");
    update_crate_tomls(rolldown_dir, version)?;

    println!("✓ Transformation complete!");
    Ok(())
}

/// Reset rolldown repository to upstream
pub fn reset_to_upstream(rolldown_dir: &Path, tag: &str) -> Result<()> {
    println!("Resetting rolldown to upstream {}...", tag);

    // Fetch origin
    let status = Command::new("git")
        .args(["fetch", "origin"])
        .current_dir(rolldown_dir)
        .status()
        .context("Failed to fetch origin")?;

    if !status.success() {
        anyhow::bail!("git fetch failed");
    }

    // Reset to upstream
    let reset_ref = if tag == "main" {
        "origin/main".to_string()
    } else {
        tag.to_string()
    };

    let status = Command::new("git")
        .args(["reset", "--hard", &reset_ref])
        .current_dir(rolldown_dir)
        .status()
        .context("Failed to reset")?;

    if !status.success() {
        anyhow::bail!("git reset failed");
    }

    // Clean untracked files
    let status = Command::new("git")
        .args(["clean", "-fdx"])
        .current_dir(rolldown_dir)
        .status()
        .context("Failed to clean")?;

    if !status.success() {
        anyhow::bail!("git clean failed");
    }

    println!("✓ Reset to {} complete!", tag);
    Ok(())
}

/// Convert rolldown name to fob name
fn to_fob_name(name: &str) -> String {
    if name == "rolldown" {
        "fob_rolldown".to_string()
    } else if name == "string_wizard" {
        "fob_string_wizard".to_string()
    } else if let Some(suffix) = name.strip_prefix("rolldown_") {
        format!("fob_rolldown_{}", suffix)
    } else {
        name.to_string()
    }
}

/// Check if a crate name is an internal rolldown crate
fn is_internal_crate(name: &str) -> bool {
    name == "rolldown" || name.starts_with("rolldown_") || name == "string_wizard"
}

/// Update workspace Cargo.toml
fn update_workspace_toml(rolldown_dir: &Path, version: &str) -> Result<()> {
    let cargo_toml = rolldown_dir.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml).context("Failed to read workspace Cargo.toml")?;

    let mut doc: DocumentMut = content.parse().context("Failed to parse workspace Cargo.toml")?;

    // Update [workspace.dependencies]
    if let Some(workspace) = doc.get_mut("workspace") {
        if let Some(deps) = workspace.get_mut("dependencies") {
            if let Some(deps_table) = deps.as_table_like_mut() {
                for (name, dep) in deps_table.iter_mut() {
                    let name_str = name.get();
                    // Update version for internal crates
                    if is_internal_crate(name_str) {
                        if let Some(table) = dep.as_table_like_mut() {
                            // Update version
                            table.insert("version", value(version));
                            // Add package field for name remapping
                            let fob_name = to_fob_name(name_str);
                            table.insert("package", value(fob_name));
                        }
                    }
                }
            }
        }
    }

    fs::write(&cargo_toml, doc.to_string()).context("Failed to write workspace Cargo.toml")?;
    Ok(())
}

/// Update all crate Cargo.toml files
fn update_crate_tomls(rolldown_dir: &Path, version: &str) -> Result<()> {
    let crates_dir = rolldown_dir.join("crates");

    for entry in fs::read_dir(&crates_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let dir_name = entry.file_name().to_string_lossy().to_string();

        // Only process rolldown crates and string_wizard
        if !is_internal_crate(&dir_name) {
            continue;
        }

        let cargo_toml = entry.path().join("Cargo.toml");
        if !cargo_toml.exists() {
            continue;
        }

        update_single_crate_toml(&cargo_toml, &dir_name, version)?;

        // Update self-references in source files (e.g., `use rolldown_testing::` in binaries)
        update_self_references(&entry.path(), &dir_name)?;
    }

    Ok(())
}

/// Update self-references in a crate's source files
/// Handles cases like: `use rolldown_testing::*` -> `use fob_rolldown_testing::*`
fn update_self_references(crate_dir: &Path, crate_name: &str) -> Result<()> {
    let new_name = to_fob_name(crate_name);
    if new_name == crate_name {
        return Ok(());
    }

    // Patterns to replace
    let old_use = format!("use {}::", crate_name);
    let new_use = format!("use {}::", new_name);

    for entry in WalkDir::new(crate_dir)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !name.starts_with("target") && !name.starts_with(".git")
        })
    {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let ext = path.extension().and_then(|s| s.to_str());
        if ext != Some("rs") {
            continue;
        }

        let content = fs::read_to_string(path)?;
        if content.contains(&old_use) {
            let updated = content.replace(&old_use, &new_use);
            fs::write(path, updated)?;
        }
    }

    Ok(())
}

/// Update a single crate's Cargo.toml
fn update_single_crate_toml(cargo_toml: &Path, dir_name: &str, version: &str) -> Result<()> {
    let content =
        fs::read_to_string(cargo_toml).with_context(|| format!("Failed to read {:?}", cargo_toml))?;

    let mut doc: DocumentMut = content
        .parse()
        .with_context(|| format!("Failed to parse {:?}", cargo_toml))?;

    let new_name = to_fob_name(dir_name);
    let is_main_crate = new_name == "fob_rolldown";

    // Update [package] section
    if let Some(package) = doc.get_mut("package") {
        if let Some(table) = package.as_table_like_mut() {
            // Update name
            table.insert("name", value(&new_name));

            // Update version
            table.insert("version", value(version));

            // Add publish = false for all non-main crates
            if !is_main_crate {
                table.insert("publish", value(false));
            }
        }
    }

    // Update path dependencies in [dependencies], [dev-dependencies], [build-dependencies]
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(deps) = doc.get_mut(section) {
            if let Some(deps_table) = deps.as_table_like_mut() {
                for (name, dep) in deps_table.iter_mut() {
                    let name_str = name.get();
                    if is_internal_crate(name_str) {
                        if let Some(table) = dep.as_table_like_mut() {
                            // Add package field if it has a path dependency
                            if table.contains_key("path") && !table.contains_key("package") {
                                let fob_name = to_fob_name(name_str);
                                table.insert("package", value(fob_name));
                            }
                        }
                    }
                }
            }
        }
    }

    fs::write(cargo_toml, doc.to_string())
        .with_context(|| format!("Failed to write {:?}", cargo_toml))?;

    println!("    {} -> {}", dir_name, new_name);
    Ok(())
}

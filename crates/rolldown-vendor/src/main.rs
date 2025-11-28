//! Rolldown Vendor - CLI tool for publishing rolldown as fob_rolldown to crates.io
//!
//! Commands:
//! - update: Reset to upstream + transform + build (for updating to new version)
//! - build: Transform + build (for re-running on already-reset repo)
//! - check: Transform + build + publish-dry (verification without reset)
//! - publish: Publish to crates.io

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod publish;
mod transform;

use publish::publish_crates;
use transform::{reset_to_upstream, transform_in_place};

/// Default rolldown directory
fn default_rolldown_dir() -> PathBuf {
    PathBuf::from("/Users/fox/src/nine-gen/rolldown")
}

#[derive(Parser)]
#[command(name = "rolldown-vendor")]
#[command(about = "Publish rolldown as fob_rolldown to crates.io")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Reset to upstream + transform + build (for updating to new rolldown version)
    Update {
        /// Version to set for all crates
        #[arg(long)]
        version: String,
        /// Git tag or branch to reset to
        #[arg(long, default_value = "main")]
        tag: String,
        /// Path to rolldown repository
        #[arg(long)]
        rolldown_dir: Option<PathBuf>,
    },

    /// Transform rolldown to fob_rolldown and build
    Build {
        /// Version to set for all crates
        #[arg(long)]
        version: String,
        /// Path to rolldown repository
        #[arg(long)]
        rolldown_dir: Option<PathBuf>,
    },

    /// Publish crates to crates.io
    Publish {
        /// Path to rolldown repository (transformed)
        #[arg(long)]
        rolldown_dir: Option<PathBuf>,
        /// Dry-run mode (verify packaging without actually publishing)
        #[arg(long)]
        dry_run: bool,
    },

    /// Transform + build + publish-dry (verification without reset)
    Check {
        /// Version to set for all crates
        #[arg(long)]
        version: String,
        /// Path to rolldown repository
        #[arg(long)]
        rolldown_dir: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Update {
            version,
            tag,
            rolldown_dir,
        } => {
            let dir = rolldown_dir.unwrap_or_else(default_rolldown_dir);

            // Step 1: Reset to upstream
            println!("=== Step 1: Reset to upstream ===");
            reset_to_upstream(&dir, &tag)?;

            // Step 2: Transform + build
            println!("\n=== Step 2: Transform + Build ===");
            transform_in_place(&dir, &version)?;

            println!("\nBuilding workspace...");
            let status = std::process::Command::new("cargo")
                .args(["build", "--workspace"])
                .current_dir(&dir)
                .status()?;

            if !status.success() {
                anyhow::bail!("Build failed");
            }
            println!("✓ Update complete!");
        }

        Command::Build {
            version,
            rolldown_dir,
        } => {
            let dir = rolldown_dir.unwrap_or_else(default_rolldown_dir);
            transform_in_place(&dir, &version)?;

            println!("\nBuilding workspace...");
            let status = std::process::Command::new("cargo")
                .args(["build", "--workspace"])
                .current_dir(&dir)
                .status()?;

            if !status.success() {
                anyhow::bail!("Build failed");
            }
            println!("✓ Build succeeded!");
        }

        Command::Publish {
            rolldown_dir,
            dry_run,
        } => {
            let dir = rolldown_dir.unwrap_or_else(default_rolldown_dir);
            publish_crates(dir.to_str().unwrap(), false, dry_run).await?;
        }

        Command::Check {
            version,
            rolldown_dir,
        } => {
            let dir = rolldown_dir.unwrap_or_else(default_rolldown_dir);

            // Step 1: Transform + build
            println!("=== Step 1: Transform + Build ===");
            transform_in_place(&dir, &version)?;

            println!("\nBuilding workspace...");
            let status = std::process::Command::new("cargo")
                .args(["build", "--workspace"])
                .current_dir(&dir)
                .status()?;

            if !status.success() {
                anyhow::bail!("Build failed");
            }
            println!("✓ Build succeeded!");

            // Step 2: Publish dry-run
            println!("\n=== Step 2: Publish (dry-run) ===");
            publish_crates(dir.to_str().unwrap(), false, true).await?;

            println!("\n✓ All checks passed! Ready to publish with: just rolldown-publish");
        }
    }

    Ok(())
}

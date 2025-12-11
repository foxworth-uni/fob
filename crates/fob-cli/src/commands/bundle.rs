use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use fob_bundler::{BuildOptions, BuildResult};
use tokio::fs;

use crate::analysis::AnalysisDocument;
use crate::cli::{BundleArgs, SourceMapSetting};
use crate::server;

pub struct BundleCommand {
    args: BundleArgs,
}

impl From<BundleArgs> for BundleCommand {
    fn from(args: BundleArgs) -> Self {
        Self { args }
    }
}

impl BundleCommand {
    pub async fn run(self) -> Result<()> {
        let entry = self
            .args
            .entries
            .first()
            .ok_or_else(|| anyhow!("At least one entry is required"))?
            .clone();

        if self.args.entries.len() > 1 {
            eprintln!(
                "{} Currently only single-entry bundling is supported; extra entries will be ignored.",
                style("[warn]").yellow()
            );
        }

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} {msg}")?.tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        );
        pb.set_message(format!("Bundling {}", entry));
        pb.enable_steady_tick(std::time::Duration::from_millis(120));

        let mut builder = BuildOptions::new_library(&entry);

        match self.args.sourcemap {
            SourceMapSetting::File => {
                builder = builder.sourcemap(true);
            }
            SourceMapSetting::Inline => {
                builder = builder.sourcemap_inline();
            }
            SourceMapSetting::Hidden => {
                builder = builder.sourcemap_hidden();
            }
            SourceMapSetting::None => {
                builder = builder.sourcemap(false);
            }
        }

        if self.args.minify {
            builder = builder.minify_level("identifiers");
        }

        let result = builder
            .build()
            .await
            .with_context(|| format!("Failed to bundle entry '{}'.", entry))?;

        let out_dir = PathBuf::from(&self.args.out_dir);
        write_assets(&result, &out_dir).await?;
        pb.finish_and_clear();

        print_summary(&result, &out_dir);

        let analysis = AnalysisDocument::from_build_result(&result);

        if let Some(path) = &self.args.json {
            write_analysis_json(path, &analysis).await?;
            println!(
                "{} Wrote analysis JSON to {}",
                style("✔").green().bold(),
                style(path).bold()
            );
        }

        if self.args.viz {
            server::serve(analysis, self.args.port).await?;
        }

        Ok(())
    }
}

async fn write_assets(result: &BuildResult, out_dir: &PathBuf) -> Result<()> {
    fs::create_dir_all(out_dir)
        .await
        .with_context(|| format!("Unable to create output directory '{}'.", out_dir.display()))?;

    // Get assets from the build result
    let bundle = result.output.as_single().expect("single bundle for library build");
    for asset in bundle.assets.iter() {
        let path = out_dir.join(asset.filename());
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .with_context(|| format!("Failed to create '{}'.", parent.display()))?;
        }
        fs::write(&path, asset.content_as_bytes())
            .await
            .with_context(|| format!("Failed to write asset '{}'.", path.display()))?;
    }

    Ok(())
}

async fn write_analysis_json(path: &str, analysis: &AnalysisDocument) -> Result<()> {
    let json = analysis.to_pretty_json()?;
    if let Some(parent) = PathBuf::from(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).await.ok();
        }
    }
    fs::write(path, json).await?;
    Ok(())
}

fn print_summary(result: &BuildResult, out_dir: &PathBuf) {
    let stats = result.stats();
    let cache = result.cache();

    println!(
        "{} Bundle complete → {}",
        style("✔").green().bold(),
        style(out_dir.display()).bold()
    );

    println!(
        "  {} modules | {} externals | {} unused exports | cache {:.1}%",
        style(stats.module_count).cyan(),
        style(stats.external_dependency_count).cyan(),
        style(stats.unused_export_count).yellow(),
        cache.hit_rate * 100.0
    );
}

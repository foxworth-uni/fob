use std::path::PathBuf;

use anyhow::{Context, Result};
use console::style;
use fob_bundler::analyze;
use fob_bundler::analysis::CacheAnalysis;
use tokio::fs;

use crate::analysis::AnalysisDocument;
use crate::cli::AnalyzeArgs;
use crate::server;

pub struct AnalyzeCommand {
    args: AnalyzeArgs,
}

impl From<AnalyzeArgs> for AnalyzeCommand {
    fn from(args: AnalyzeArgs) -> Self {
        Self { args }
    }
}

impl AnalyzeCommand {
    pub async fn run(self) -> Result<()> {
        let entry_paths: Vec<PathBuf> = self.args.entries.iter().map(PathBuf::from).collect();

        let analysis = analyze(entry_paths.clone())
            .await
            .context("Static analysis failed")?;

        let document = AnalysisDocument::from_analysis(&analysis, CacheAnalysis::default());

        println!(
            "{} Analysis complete for {} entry(ies)",
            style("✔").green().bold(),
            style(entry_paths.len()).cyan()
        );

        if let Some(path) = &self.args.json {
            write_analysis_json(path, &document).await?;
            println!(
                "{} Wrote analysis JSON to {}",
                style("✔").green().bold(),
                style(path).bold()
            );
        }

        if self.args.viz {
            server::serve(document, self.args.port).await?;
        }

        Ok(())
    }
}

async fn write_analysis_json(path: &str, analysis: &AnalysisDocument) -> Result<()> {
    let json = analysis.to_pretty_json()?;
    let path_buf = PathBuf::from(path);
    if let Some(parent) = path_buf.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).await.ok();
        }
    }
    fs::write(path_buf, json.into_bytes()).await?;
    Ok(())
}

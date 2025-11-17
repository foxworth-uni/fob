use std::path::PathBuf;

use anyhow::{Context, Result};
use console::style;
use tokio::fs;

use crate::analysis::AnalysisDocument;
use crate::cli::ServeArgs;
use crate::server;

pub struct ServeCommand {
    args: ServeArgs,
}

impl From<ServeArgs> for ServeCommand {
    fn from(args: ServeArgs) -> Self {
        Self { args }
    }
}

impl ServeCommand {
    pub async fn run(self) -> Result<()> {
        let path = PathBuf::from(&self.args.analysis);
        let contents = fs::read_to_string(&path)
            .await
            .with_context(|| format!("Failed to read analysis file '{}'.", path.display()))?;

        let document: AnalysisDocument = serde_json::from_str(&contents)
            .with_context(|| "Invalid analysis JSON format".to_string())?;

        println!(
            "{} Serving analysis dashboard from {}",
            style("ℹ").cyan(),
            style(path.display()).bold()
        );

        server::serve(document, self.args.port).await?;
        Ok(())
    }
}

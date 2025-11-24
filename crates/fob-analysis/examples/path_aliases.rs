//! Path alias configuration example.
//!
//! This example shows how to configure and use path aliases for import resolution.

use fob_analysis::Analyzer;

#[tokio::main]
async fn main() -> fob::Result<()> {
    // Configure path aliases
    // Common patterns:
    // - "@" → "./src" (TypeScript/JavaScript convention)
    // - "~" → "./src" (alternative convention)
    // - "#" → "./src" (another alternative)
    let analysis = Analyzer::new()
        .entry("src/index.ts")
        .path_alias("@", "./src")  // "@/components/Button" → "./src/components/Button"
        .path_alias("~", "./src")  // "~/utils/helpers" → "./src/utils/helpers"
        .external(vec!["react", "lodash"])  // Mark npm packages as external
        .analyze()
        .await?;

    println!("Analysis completed with path aliases configured");
    println!("{}", analysis);

    Ok(())
}


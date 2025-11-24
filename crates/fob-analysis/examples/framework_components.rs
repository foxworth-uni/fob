//! Framework-specific component analysis example.
//!
//! This example shows how to analyze framework components (Astro, Svelte, Vue)
//! which require script extraction before parsing.

use fob_analysis::{Analyzer, AnalyzeOptions};

#[tokio::main]
async fn main() -> fob::Result<()> {
    // Analyze framework components
    // The analyzer automatically extracts JavaScript/TypeScript from:
    // - Astro components (.astro)
    // - Svelte components (.svelte)
    // - Vue Single File Components (.vue)
    let analysis = Analyzer::new()
        .entries(vec![
            "src/components/App.astro",      // Astro component
            "src/components/Button.svelte",  // Svelte component
            "src/components/Card.vue",       // Vue component
        ])
        .external(vec!["react", "vue", "svelte"])  // Mark frameworks as external
        .analyze()
        .await?;

    println!("Framework components analyzed successfully!");
    println!("{}", analysis);

    // You can also apply framework-specific rules to mark exports as framework-used
    // This prevents false-positive "unused export" warnings for framework conventions
    let options = AnalyzeOptions {
        framework_rules: vec![
            // Add your custom framework rules here
            // Example: Box::new(MyReactRule),
        ],
        compute_usage_counts: true,
    };

    let _analysis_with_rules = Analyzer::new()
        .entry("src/components/App.astro")
        .analyze_with_options(options)
        .await?;

    println!("\nAnalysis with framework rules completed!");

    Ok(())
}


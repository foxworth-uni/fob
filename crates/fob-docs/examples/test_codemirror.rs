//! Test LLM enhancement on the CodeMirror bundle demo index.js file
//!
//! Run with:
//! cargo run --package fob-docs --features llm --example test_codemirror

#[cfg(feature = "llm")]
use fob_docs::{
    llm::{EnhancementMode, LlmConfig, LlmEnhancer},
    DocsExtractor, ExtractOptions,
};

#[cfg(feature = "llm")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Testing LLM Enhancement on CodeMirror Demo ===\n");

    let file_path = "examples/js/test-llm-enhancement.js";

    println!("Reading file: {}\n", file_path);

    // Extract documentation from the file
    let extractor = DocsExtractor::new(ExtractOptions::default());
    let module = extractor.extract_from_path(file_path)?;

    println!("Extracted documentation:");
    println!("  Module: {}", module.path);
    println!("  Symbols found: {}", module.symbols.len());

    for symbol in &module.symbols {
        println!("    - {} ({})", symbol.name, format!("{:?}", symbol.kind));
        if let Some(summary) = &symbol.summary {
            println!("      Summary: {}", summary);
        } else {
            println!("      Summary: <none - will be enhanced>");
        }
    }

    // Check if we have any symbols to enhance
    let total_symbols = module.symbols.len();

    if total_symbols == 0 {
        println!("\n⚠️  No exported symbols found in the file.");
        println!("This is expected - the CodeMirror demo doesn't export anything.");
        println!("It's just initialization code.");
        return Ok(());
    }

    // Create Documentation from the module
    let mut docs = fob_docs::Documentation::default();
    docs.add_module(module);

    println!("\n--- Setting up LLM Enhancement ---\n");

    // Configure LLM enhancement (Missing mode = only enhance undocumented symbols)
    let config = LlmConfig::default()
        .with_model("llama3.2:3b")
        .with_mode(EnhancementMode::Missing);

    println!("LLM Configuration:");
    println!("  Provider: {}", config.provider);
    println!("  Model: {}", config.model);
    println!("  Mode: {} (only enhance symbols with no JSDoc)", config.enhancement_mode);
    println!("  Cache: {}", config.cache_enabled);

    println!("\n🚀 Initializing LLM enhancer...");

    let enhancer = match LlmEnhancer::new(config).await {
        Ok(e) => {
            println!("✅ LLM enhancer initialized successfully!\n");
            e
        }
        Err(err) => {
            eprintln!("\n❌ Error initializing LLM enhancer: {}", err);
            eprintln!("\n💡 Troubleshooting:");
            eprintln!("  1. Is Ollama running? Try: ollama serve");
            eprintln!("  2. Do you have the model? Try: ollama pull llama3.2:3b");
            eprintln!("  3. Check Ollama at http://localhost:11434");
            return Err(err.into());
        }
    };

    println!("🤖 Enhancing documentation with AI...\n");

    // Enhance documentation
    let enhanced = enhancer
        .enhance_documentation(docs, |current, total| {
            println!("  Progress: {}/{} symbols enhanced", current, total);
        })
        .await?;

    println!("\n✨ Enhancement complete!\n");
    println!("=== Enhanced Documentation ===\n");

    for module in &enhanced.modules {
        println!("Module: {}", module.path);

        for symbol in &module.symbols {
            println!("\n  📦 Symbol: {}", symbol.name);
            println!("  Kind: {:?}", symbol.kind);

            if let Some(summary) = &symbol.summary {
                println!("  📝 Summary:");
                for line in summary.lines() {
                    println!("     {}", line);
                }
            }

            if !symbol.examples.is_empty() {
                println!("  💡 Examples:");
                for example in &symbol.examples {
                    println!("     {}", example);
                }
            }
        }
    }

    println!("\n=== Test Complete ===");

    Ok(())
}

#[cfg(not(feature = "llm"))]
fn main() {
    eprintln!("This example requires the 'llm' feature to be enabled.");
    eprintln!("Run with: cargo run --package fob-docs --features llm --example test_codemirror");
    std::process::exit(1);
}

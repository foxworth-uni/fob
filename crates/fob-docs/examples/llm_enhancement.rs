//! Example: Using LLM enhancement for documentation generation
//!
//! This example shows how to use the LLM enhancement feature to automatically
//! generate high-quality documentation for TypeScript/JavaScript code.
//!
//! Prerequisites:
//! 1. Install Ollama: https://ollama.com
//! 2. Pull a model: `ollama pull llama3.2:3b`
//! 3. Start Ollama: `ollama serve`
//!
//! Run this example:
//! ```bash
//! cargo run --package fob-docs --features llm --example llm_enhancement
//! ```

#[cfg(feature = "llm")]
use fob_docs::{
    llm::{EnhancementMode, LlmConfig, LlmEnhancer},
    Documentation, ExportedSymbol, ModuleDoc, ParameterDoc, SourceLocation, SymbolKind,
};

#[cfg(feature = "llm")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== LLM-Enhanced Documentation Example ===\n");

    // Create sample documentation (in real usage, this comes from DocsExtractor)
    let mut docs = Documentation::default();

    let mut symbol = ExportedSymbol::new(
        "calculateTotal",
        SymbolKind::Function,
        SourceLocation::new(10, 1),
    );

    symbol.parameters.push(ParameterDoc {
        name: "items".to_string(),
        type_hint: Some("number[]".to_string()),
        description: None, // No description - LLM will add one
    });

    symbol.returns = Some("number".to_string());
    // No summary - this is what the LLM will generate

    let mut module = ModuleDoc::new("src/math.ts");
    module.symbols.push(symbol);
    docs.add_module(module);

    println!("Original documentation (without LLM):");
    println!("  Symbol: calculateTotal");
    println!("  Summary: None");
    println!("  Examples: None\n");

    // Configure LLM enhancement
    let config = LlmConfig::default()
        .with_model("llama3.2:3b")
        .with_mode(EnhancementMode::Missing);

    println!("LLM Configuration:");
    println!("  Provider: {}", config.provider);
    println!("  Model: {}", config.model);
    println!("  Mode: {}", config.enhancement_mode);
    println!("  Cache: {}\n", config.cache_enabled);

    // Create enhancer
    println!("Initializing LLM enhancer...");
    let enhancer = match LlmEnhancer::new(config).await {
        Ok(e) => e,
        Err(err) => {
            eprintln!("\nError: {}", err);
            eprintln!("\nTroubleshooting:");
            eprintln!("  1. Is Ollama running? Try: ollama serve");
            eprintln!("  2. Do you have the model? Try: ollama pull llama3.2:3b");
            eprintln!("  3. Check Ollama is accessible at http://localhost:11434");
            return Err(err.into());
        }
    };

    println!("\nEnhancing documentation...");

    // Enhance documentation
    let enhanced = enhancer
        .enhance_documentation(docs, |current, total| {
            println!("  Progress: {}/{} symbols", current, total);
        })
        .await?;

    println!("\nEnhanced documentation (with LLM):");
    for module in &enhanced.modules {
        println!("\nModule: {}", module.path);
        for symbol in &module.symbols {
            println!("  Symbol: {}", symbol.name);
            if let Some(summary) = &symbol.summary {
                println!("  Summary: {}", summary);
            }
            if !symbol.examples.is_empty() {
                println!("  Examples:");
                for example in &symbol.examples {
                    for line in example.lines() {
                        println!("    {}", line);
                    }
                }
            }
        }
    }

    println!("\n=== Enhancement Complete ===");
    println!("\nNext steps:");
    println!("  - Try different modes: --mode=incomplete or --mode=all");
    println!("  - Use different models: --model=codellama:7b");
    println!("  - Check the cache: .fob-cache/docs-llm/");

    Ok(())
}

#[cfg(not(feature = "llm"))]
fn main() {
    eprintln!("This example requires the 'llm' feature to be enabled.");
    eprintln!("Run with: cargo run --package fob-docs --features llm --example llm_enhancement");
    std::process::exit(1);
}

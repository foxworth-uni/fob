//! Symbol table usage and filtering example.
//!
//! This example demonstrates:
//! - Analyzing JavaScript/TypeScript source code to extract symbols
//! - Finding unused symbols
//! - Filtering symbols by kind
//! - Accessing symbol metadata (read/write counts, spans)

use fob_graph::semantic::analyze_symbols;
use fob_graph::{SourceType, SymbolKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example TypeScript source code
    let source = r#"
        // Unused variable
        const unusedVar = 42;

        // Used variable
        const usedVar = 100;
        console.log(usedVar);

        // Function declaration
        function calculateSum(a: number, b: number): number {
            return a + b;
        }

        // Used function
        const result = calculateSum(5, 10);

        // Unused function
        function unusedFunction() {
            return "never called";
        }

        // Class declaration
        class MyClass {
            public method(): void {
                console.log("method called");
            }
        }

        // Used class
        const instance = new MyClass();
        instance.method();
    "#;

    // Analyze the source code
    let table = analyze_symbols(source, "example.ts", SourceType::TypeScript)?;

    println!("Total symbols found: {}", table.symbols.len());
    println!("\nAll symbols:");
    for symbol in &table.symbols {
        println!(
            "  {} ({:?}) - reads: {}, writes: {}",
            symbol.name, symbol.kind, symbol.read_count, symbol.write_count
        );
    }

    // Find unused symbols (symbols with zero reads and writes)
    let unused = table.unused_symbols();
    println!("\nUnused symbols ({}):", unused.len());
    for symbol in unused {
        println!("  - {} ({:?})", symbol.name, symbol.kind);
    }

    // Filter symbols by kind
    println!("\nFunctions:");
    let functions: Vec<_> = table
        .symbols
        .iter()
        .filter(|s| s.kind == SymbolKind::Function)
        .collect();
    for func in functions {
        println!("  - {} (reads: {})", func.name, func.read_count);
    }

    println!("\nClasses:");
    let classes: Vec<_> = table
        .symbols
        .iter()
        .filter(|s| s.kind == SymbolKind::Class)
        .collect();
    for class in classes {
        println!("  - {} (reads: {})", class.name, class.read_count);
    }

    println!("\nVariables:");
    let variables: Vec<_> = table
        .symbols
        .iter()
        .filter(|s| s.kind == SymbolKind::Variable)
        .collect();
    for var in variables {
        println!(
            "  - {} (reads: {}, writes: {})",
            var.name, var.read_count, var.write_count
        );
    }

    // Access symbol spans (line/column information)
    println!("\nSymbol locations:");
    for symbol in &table.symbols {
        let span = &symbol.declaration_span;
        println!(
            "  {} at line {}, column {}",
            symbol.name, span.line, span.column
        );
    }

    Ok(())
}


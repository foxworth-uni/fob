// Simple test to verify semantic analysis works
use fob_core::graph::semantic::analyze_symbols;
use fob_core::graph::SourceType;

fn main() {
    println!("Testing semantic analysis implementation...\n");

    // Test 1: Simple variable usage
    let source1 = r#"
        const used = 42;
        const unused = 100;
        console.log(used);
    "#;

    println!("Test 1: Simple variables");
    match analyze_symbols(source1, "test.js", SourceType::JavaScript) {
        Ok(table) => {
            println!("  ✓ Symbols found: {}", table.symbols.len());
            for symbol in &table.symbols {
                println!("    - {} ({:?}): reads={}, writes={}",
                    symbol.name, symbol.kind, symbol.read_count, symbol.write_count);
            }

            let used = table.symbols_by_name("used");
            let unused = table.symbols_by_name("unused");

            if !used.is_empty() && used[0].read_count > 0 {
                println!("  ✓ 'used' variable detected with reads");
            }
            if !unused.is_empty() && unused[0].read_count == 0 {
                println!("  ✓ 'unused' variable detected with no reads");
            }
        }
        Err(e) => println!("  ✗ Error: {}", e),
    }

    println!();

    // Test 2: Function usage
    let source2 = r#"
        function usedFunction() {
            return 42;
        }

        function unusedFunction() {
            return 100;
        }

        usedFunction();
    "#;

    println!("Test 2: Functions");
    match analyze_symbols(source2, "test.js", SourceType::JavaScript) {
        Ok(table) => {
            println!("  ✓ Symbols found: {}", table.symbols.len());
            for symbol in &table.symbols {
                println!("    - {} ({:?}): reads={}, writes={}",
                    symbol.name, symbol.kind, symbol.read_count, symbol.write_count);
            }

            let used_fn = table.symbols_by_name("usedFunction");
            let unused_fn = table.symbols_by_name("unusedFunction");

            if !used_fn.is_empty() && used_fn[0].read_count > 0 {
                println!("  ✓ 'usedFunction' detected with reads");
            }
            if !unused_fn.is_empty() && unused_fn[0].read_count == 0 {
                println!("  ✓ 'unusedFunction' detected with no reads");
            }
        }
        Err(e) => println!("  ✗ Error: {}", e),
    }

    println!();

    // Test 3: TypeScript types
    let source3 = r#"
        interface User {
            name: string;
        }

        type UserId = string;

        const user: User = { name: "test" };
    "#;

    println!("Test 3: TypeScript");
    match analyze_symbols(source3, "test.ts", SourceType::TypeScript) {
        Ok(table) => {
            println!("  ✓ Symbols found: {}", table.symbols.len());
            println!("  ✓ Scopes found: {}", table.scope_count);
            for symbol in &table.symbols {
                println!("    - {} ({:?}): reads={}, writes={}",
                    symbol.name, symbol.kind, symbol.read_count, symbol.write_count);
            }
        }
        Err(e) => println!("  ✗ Error: {}", e),
    }

    println!();

    // Test 4: Parse error handling
    let source4 = "const x = {{{{{ invalid syntax";

    println!("Test 4: Parse error handling");
    match analyze_symbols(source4, "invalid.js", SourceType::JavaScript) {
        Ok(table) => {
            if table.is_empty() {
                println!("  ✓ Gracefully handled parse error (empty table)");
            } else {
                println!("  ✗ Expected empty table for invalid syntax");
            }
        }
        Err(e) => println!("  ✗ Should not error, should return empty table: {}", e),
    }

    println!();

    // Test 5: Unreachable code detection
    let source5 = r#"
        function example() {
            return true;
            console.log('unreachable');
        }

        function another() {
            throw new Error('error');
            console.log('also unreachable');
        }
    "#;

    println!("Test 5: Unreachable code detection");
    use fob_core::graph::ModuleId;
    let module_id = ModuleId::new("test.js").unwrap();
    match fob_core::graph::semantic::detect_unreachable_code(
        source5,
        "test.js",
        SourceType::JavaScript,
        module_id,
    ) {
        Ok(unreachable) => {
            if unreachable.len() > 0 {
                println!("  ✓ Detected {} unreachable code blocks", unreachable.len());
                for code in &unreachable {
                    println!("    - Line {}: {}", code.span.line, code.description);
                }
            } else {
                println!("  ✗ Expected to find unreachable code");
            }
        }
        Err(e) => println!("  ✗ Error: {}", e),
    }

    println!("\nAll tests complete!");
}

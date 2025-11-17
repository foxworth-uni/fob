use fob_docs::{
    generators::json::render_json, generators::markdown::render_markdown, DocsExtractor,
    Documentation, ExtractOptions,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        /**
         * Subtract the second value from the first.
         * @param {number} a minuend
         * @param {number} b subtrahend
         * @returns {number} difference
         */
        export function subtract(a: number, b: number): number {
            return a - b;
        }
    "#;

    let extractor = DocsExtractor::new(ExtractOptions::default());
    let module = extractor.extract_from_source("src/math.ts", source)?;

    let mut documentation = Documentation::default();
    documentation.add_module(module);

    let markdown = render_markdown(&documentation);
    println!("Markdown output:\n{markdown}");

    let json = render_json(&documentation)?;
    println!("JSON output:\n{json}");

    Ok(())
}

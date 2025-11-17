//! Example: Using HtmlBuilder for dev server HTML generation

use fob_gen::{Allocator, HtmlBuilder, RouteSpec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let allocator = Allocator::default();
    let html = HtmlBuilder::new(&allocator);

    // Generate index.html
    println!("=== Index HTML ===");
    let index_html = html.index_html(Some("/index.js"))?;
    println!("{}", index_html);
    println!();

    // Generate error overlay
    println!("=== Error Overlay ===");
    let error_html = html.error_overlay("Failed to compile: SyntaxError: Unexpected token")?;
    println!("{}", error_html);
    println!();

    // Generate route manifest
    println!("=== Route Manifest ===");
    let routes = vec![
        RouteSpec {
            path: "/".to_string(),
            id: "index".to_string(),
            file: "./routes/index.tsx".to_string(),
        },
        RouteSpec {
            path: "/about".to_string(),
            id: "about".to_string(),
            file: "./routes/about.tsx".to_string(),
        },
        RouteSpec {
            path: "/blog/:slug".to_string(),
            id: "blog_post".to_string(),
            file: "./routes/blog/[slug].tsx".to_string(),
        },
    ];
    let manifest = html.route_manifest(&routes)?;
    println!("{}", manifest);

    Ok(())
}


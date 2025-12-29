//! Example: Generating a route manifest for a web framework
//!
//! This demonstrates how `fob-gen` can generate complex JavaScript objects
//! for configuration and routing.

use fob_gen::{Allocator, ProgramBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let allocator = Allocator::default();
    let mut js = ProgramBuilder::new(&allocator);

    // Create route entries
    let routes = vec![
        ("/", "index", "./routes/index.tsx"),
        ("/about", "about", "./routes/about.tsx"),
        ("/blog/:slug", "blog_post", "./routes/blog/[slug].tsx"),
    ];

    let route_objects: Vec<_> = routes
        .into_iter()
        .map(|(path, id, file)| {
            js.object(vec![
                js.prop("path", js.string(path)),
                js.prop("id", js.string(id)),
                js.prop(
                    "component",
                    js.call(
                        js.ident("lazy"),
                        vec![js.arg(js.arrow_fn(
                            vec![],
                            js.call(js.ident("import"), vec![js.arg(js.string(file))]),
                        ))],
                    ),
                ),
            ])
        })
        .collect();

    // Create the manifest: const routes = [...]
    let routes_array = js.array(route_objects);
    let routes_decl = js.const_decl("routes", routes_array);

    // export default routes
    let export_default = js.export_default(js.ident("routes"));

    // Generate module
    js.extend(vec![routes_decl, Statement::from(export_default)]);
    let code = js.generate(&Default::default())?;

    println!("{}", code);
    Ok(())
}

use oxc_ast::ast::Statement;

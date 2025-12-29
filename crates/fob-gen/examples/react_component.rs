//! Example: Building a React component with fob-gen
//!
//! This demonstrates generating a complete React component with JSX.

use fob_gen::{Allocator, JsxBuilder, ProgramBuilder};
use oxc_ast::ast::Statement;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let allocator = Allocator::default();
    let mut js = ProgramBuilder::new(&allocator);
    let jsx = JsxBuilder::new(&allocator);

    // import React from 'react'
    let import_react = js.import_default("React", "react");

    // const Button = ({ label, onClick }) => (
    //   <button onClick={onClick} className="btn">
    //     {label}
    //   </button>
    // )
    let button_jsx = jsx.element(
        "button",
        vec![
            jsx.attr("onClick", Some(jsx.expr_attr(js.ident("onClick")))),
            jsx.attr("className", Some(jsx.string_attr("btn"))),
        ],
        vec![jsx.expr_child(js.ident("label"))],
        false,
    );

    let button_fn = js.arrow_fn_block(
        vec!["props"],
        vec![js.return_stmt(Some(jsx.jsx_expr(button_jsx)))],
    );

    let button_decl = js.const_decl("Button", button_fn);

    // export default Button
    let export_default = js.export_default(js.ident("Button"));

    // Generate complete module
    js.extend(vec![
        Statement::from(import_react),
        button_decl,
        Statement::from(export_default),
    ]);
    let code = js.generate(&Default::default())?;

    println!("{}", code);
    Ok(())
}

//! Comprehensive JSX builder tests

use fob_gen::{Allocator, JsBuilder, JsxBuilder};
use oxc_ast::ast::Statement;

#[test]
fn test_jsx_simple_element() {
    let allocator = Allocator::default();
    let jsx = JsxBuilder::new(&allocator);

    // <div>Hello</div>
    let element = jsx.element("div", vec![], vec![jsx.text("Hello")], false);

    let js = JsBuilder::new(&allocator);
    let stmt = js.const_decl("el", jsx.jsx_expr(element));
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("<div"));
    assert!(code.contains("</div>"));
    assert!(code.contains("Hello"));
}

#[test]
fn test_jsx_self_closing() {
    let allocator = Allocator::default();
    let jsx = JsxBuilder::new(&allocator);

    // <img />
    let element = jsx.element(
        "img",
        vec![],
        vec![],
        true, // self_closing
    );

    let js = JsBuilder::new(&allocator);
    let stmt = js.const_decl("img", jsx.jsx_expr(element));
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("<img"));
    assert!(code.contains("/>"));
    assert!(!code.contains("</img>"));
}

#[test]
fn test_jsx_with_attributes() {
    let allocator = Allocator::default();
    let jsx = JsxBuilder::new(&allocator);

    // <div className="container" id="main">Content</div>
    let element = jsx.element(
        "div",
        vec![
            jsx.attr("className", Some(jsx.string_attr("container"))),
            jsx.attr("id", Some(jsx.string_attr("main"))),
        ],
        vec![jsx.text("Content")],
        false,
    );

    let js = JsBuilder::new(&allocator);
    let stmt = js.const_decl("el", jsx.jsx_expr(element));
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("<div"));
    assert!(code.contains("className"));
    assert!(code.contains("container"));
    assert!(code.contains("id"));
    assert!(code.contains("main"));
    assert!(code.contains("Content"));
}

#[test]
fn test_jsx_boolean_attribute() {
    let allocator = Allocator::default();
    let jsx = JsxBuilder::new(&allocator);

    // <input disabled />
    let element = jsx.element("input", vec![jsx.attr("disabled", None)], vec![], true);

    let js = JsBuilder::new(&allocator);
    let stmt = js.const_decl("input", jsx.jsx_expr(element));
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("<input"));
    assert!(code.contains("disabled"));
}

#[test]
fn test_jsx_expression_attribute() {
    let allocator = Allocator::default();
    let jsx = JsxBuilder::new(&allocator);
    let js = JsBuilder::new(&allocator);

    // <div className={styles.container}>Text</div>
    let expr = js.member(js.ident("styles"), "container");
    let element = jsx.element(
        "div",
        vec![jsx.attr("className", Some(jsx.expr_attr(expr)))],
        vec![jsx.text("Text")],
        false,
    );

    let stmt = js.const_decl("el", jsx.jsx_expr(element));
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("<div"));
    assert!(code.contains("className"));
    assert!(code.contains("styles.container"));
}

#[test]
fn test_jsx_nested_elements() {
    let allocator = Allocator::default();
    let jsx = JsxBuilder::new(&allocator);

    // <div><span>Nested</span></div>
    let span = jsx.element("span", vec![], vec![jsx.text("Nested")], false);

    let div = jsx.element("div", vec![], vec![jsx.child(span)], false);

    let js = JsBuilder::new(&allocator);
    let stmt = js.const_decl("el", jsx.jsx_expr(div));
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("<div"));
    assert!(code.contains("<span"));
    assert!(code.contains("Nested"));
    assert!(code.contains("</span>"));
    assert!(code.contains("</div>"));
}

#[test]
fn test_jsx_expression_children() {
    let allocator = Allocator::default();
    let jsx = JsxBuilder::new(&allocator);
    let js = JsBuilder::new(&allocator);

    // <div>{count}</div>
    let element = jsx.element(
        "div",
        vec![],
        vec![jsx.expr_child(js.ident("count"))],
        false,
    );

    let stmt = js.const_decl("el", jsx.jsx_expr(element));
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("<div"));
    assert!(code.contains("{"));
    assert!(code.contains("count"));
    assert!(code.contains("}"));
}

#[test]
fn test_jsx_fragment() {
    let allocator = Allocator::default();
    let jsx = JsxBuilder::new(&allocator);

    // <><div>First</div><div>Second</div></>
    let first = jsx.element("div", vec![], vec![jsx.text("First")], false);
    let second = jsx.element("div", vec![], vec![jsx.text("Second")], false);

    let fragment = jsx.fragment(vec![jsx.child(first), jsx.child(second)]);

    let js = JsBuilder::new(&allocator);
    let stmt = js.const_decl("el", jsx.jsx_fragment_expr(fragment));
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("<>"));
    assert!(code.contains("</>"));
    assert!(code.contains("First"));
    assert!(code.contains("Second"));
}

#[test]
fn test_jsx_component() {
    let allocator = Allocator::default();
    let jsx = JsxBuilder::new(&allocator);

    // <Button onClick={handleClick}>Click me</Button>
    let js = JsBuilder::new(&allocator);
    let element = jsx.element(
        "Button",
        vec![jsx.attr("onClick", Some(jsx.expr_attr(js.ident("handleClick"))))],
        vec![jsx.text("Click me")],
        false,
    );

    let stmt = js.const_decl("btn", jsx.jsx_expr(element));
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("<Button"));
    assert!(code.contains("onClick"));
    assert!(code.contains("handleClick"));
    assert!(code.contains("Click me"));
    assert!(code.contains("</Button>"));
}

#[test]
fn test_jsx_mixed_children() {
    let allocator = Allocator::default();
    let jsx = JsxBuilder::new(&allocator);
    let js = JsBuilder::new(&allocator);

    // <div>Text {variable} <span>more</span></div>
    let span = jsx.element("span", vec![], vec![jsx.text("more")], false);
    let element = jsx.element(
        "div",
        vec![],
        vec![
            jsx.text("Text "),
            jsx.expr_child(js.ident("variable")),
            jsx.text(" "),
            jsx.child(span),
        ],
        false,
    );

    let stmt = js.const_decl("el", jsx.jsx_expr(element));
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("<div"));
    assert!(code.contains("Text"));
    assert!(code.contains("variable"));
    assert!(code.contains("<span"));
    assert!(code.contains("more"));
}

#[test]
fn test_jsx_in_export() {
    let allocator = Allocator::default();
    let jsx = JsxBuilder::new(&allocator);
    let js = JsBuilder::new(&allocator);

    // export default <App />
    let element = jsx.element("App", vec![], vec![], true);
    let export = js.export_default(jsx.jsx_expr(element));

    let code = js.program(vec![Statement::from(export)]).unwrap();

    assert!(code.contains("export default"));
    assert!(code.contains("<App"));
}

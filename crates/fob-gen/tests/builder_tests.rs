//! Comprehensive tests for fob-gen builders

use fob_gen::{Allocator, BinaryOperator, JsBuilder, LogicalOperator};
use oxc_ast::ast::Statement;

#[test]
fn test_primitives() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // Test number
    let stmt = js.const_decl("x", js.number(42.0));
    let code = js.program(vec![stmt]).unwrap();
    assert!(code.contains("const x = 42"));

    // Test string
    let stmt = js.const_decl("msg", js.string("hello"));
    let code = js.program(vec![stmt]).unwrap();
    assert!(code.contains(r#"const msg = "hello""#));

    // Test boolean
    let stmt = js.const_decl("flag", js.bool(true));
    let code = js.program(vec![stmt]).unwrap();
    assert!(code.contains("const flag = true"));

    // Test null
    let stmt = js.const_decl("empty", js.null());
    let code = js.program(vec![stmt]).unwrap();
    assert!(code.contains("const empty = null"));
}

#[test]
fn test_arrays() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    let arr = js.array(vec![js.number(1.0), js.number(2.0), js.number(3.0)]);
    let stmt = js.const_decl("nums", arr);
    let code = js.program(vec![stmt]).unwrap();

    // Just verify it contains the basic structure
    assert!(code.contains("const nums"));
    assert!(code.contains("["));
    assert!(code.contains("1"));
    assert!(code.contains("2"));
    assert!(code.contains("3"));
}

#[test]
fn test_objects() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    let obj = js.object(vec![
        js.prop("name", js.string("John")),
        js.prop("age", js.number(30.0)),
    ]);
    let stmt = js.const_decl("person", obj);
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("const person"));
    assert!(code.contains(r#"name: "John""#));
    assert!(code.contains("age: 30"));
}

#[test]
fn test_member_access() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // console.log
    let member = js.member(js.ident("console"), "log");
    let call = js.call(member, vec![js.arg(js.string("test"))]);
    let stmt = js.expr_stmt(call);
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("console.log"));
    assert!(code.contains(r#""test""#));
}

#[test]
fn test_computed_member() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // arr[0]
    let computed = js.computed_member(js.ident("arr"), js.number(0.0));
    let stmt = js.const_decl("first", computed);
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("const first = arr[0]"));
}

#[test]
fn test_arrow_functions() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // const double = x => x * 2
    let arrow = js.arrow_fn(
        vec!["x"],
        js.binary(
            js.ident("x"),
            BinaryOperator::Multiplication,
            js.number(2.0),
        ),
    );
    let stmt = js.const_decl("double", arrow);
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("const double"));
    assert!(code.contains("=>"));
    assert!(code.contains("x * 2"));
}

#[test]
fn test_imports() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // import React from 'react'
    let import_decl = js.import_default("React", "react");
    let stmt = Statement::from(import_decl);
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("import React from"));
    assert!(code.contains(r#""react""#));
}

#[test]
fn test_named_imports() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // import { useState, useEffect } from 'react'
    let import_decl = js.import_named(vec!["useState", "useEffect"], "react");
    let stmt = Statement::from(import_decl);
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("import"));
    assert!(code.contains("useState"));
    assert!(code.contains("useEffect"));
    assert!(code.contains(r#""react""#));
}

#[test]
fn test_exports() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // export const greeting = "Hello"
    let export_decl = js.export_const("greeting", js.string("Hello"));
    let stmt = Statement::from(export_decl);
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("export"));
    assert!(code.contains("const greeting"));
    assert!(code.contains(r#""Hello""#));
}

#[test]
fn test_export_default() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // export default 42
    let export_decl = js.export_default(js.number(42.0));
    let stmt = Statement::from(export_decl);
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("export default 42"));
}

#[test]
fn test_if_statement() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // if (x > 0) { return true; } else { return false; }
    let test = js.binary(js.ident("x"), BinaryOperator::GreaterThan, js.number(0.0));
    let consequent = vec![js.return_stmt(Some(js.bool(true)))];
    let alternate = vec![js.return_stmt(Some(js.bool(false)))];

    let if_stmt = js.if_stmt(test, consequent, Some(alternate));
    let code = js.program(vec![if_stmt]).unwrap();

    assert!(code.contains("if"));
    assert!(code.contains("x > 0"));
    assert!(code.contains("return true"));
    assert!(code.contains("return false"));
}

#[test]
fn test_logical_operators() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // const result = a && b || c
    let expr = js.logical(
        js.logical(js.ident("a"), LogicalOperator::And, js.ident("b")),
        LogicalOperator::Or,
        js.ident("c"),
    );
    let stmt = js.const_decl("result", expr);
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("const result"));
    assert!(code.contains("a && b"));
    assert!(code.contains("||"));
    assert!(code.contains("c"));
}

#[test]
fn test_conditional_expression() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // const result = x > 0 ? "positive" : "negative"
    let test = js.binary(js.ident("x"), BinaryOperator::GreaterThan, js.number(0.0));
    let conditional = js.conditional(test, js.string("positive"), js.string("negative"));
    let stmt = js.const_decl("result", conditional);
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("const result"));
    assert!(code.contains("x > 0"));
    assert!(code.contains("?"));
    assert!(code.contains(r#""positive""#));
    assert!(code.contains(r#""negative""#));
}

#[test]
fn test_new_expression() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // const err = new Error("Failed")
    let new_expr = js.new_expr(js.ident("Error"), vec![js.arg(js.string("Failed"))]);
    let stmt = js.const_decl("err", new_expr);
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("const err = new Error"));
    assert!(code.contains(r#""Failed""#));
}

#[test]
fn test_not_operator() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // const result = !flag
    let not_expr = js.not(js.ident("flag"));
    let stmt = js.const_decl("result", not_expr);
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("const result = !flag"));
}

#[test]
fn test_complex_program() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // import React from 'react';
    // const greeting = "Hello";
    // console.log(greeting);
    let import_stmt = Statement::from(js.import_default("React", "react"));
    let const_decl = js.const_decl("greeting", js.string("Hello"));
    let console_log = js.call(
        js.member(js.ident("console"), "log"),
        vec![js.arg(js.ident("greeting"))],
    );
    let expr_stmt = js.expr_stmt(console_log);

    let code = js
        .program(vec![import_stmt, const_decl, expr_stmt])
        .unwrap();

    assert!(code.contains("import React"));
    assert!(code.contains("const greeting"));
    assert!(code.contains("console.log"));
}

#[test]
fn test_let_declaration() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // let x;
    let stmt = js.let_decl("x", None);
    let code = js.program(vec![stmt]).unwrap();
    assert!(code.contains("let x"));

    // let y = 10;
    let stmt = js.let_decl("y", Some(js.number(10.0)));
    let code = js.program(vec![stmt]).unwrap();
    assert!(code.contains("let y = 10"));
}

#[test]
fn test_throw_statement() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // throw new Error("Oops")
    let error = js.new_expr(js.ident("Error"), vec![js.arg(js.string("Oops"))]);
    let stmt = js.throw(error);
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("throw new Error"));
    assert!(code.contains(r#""Oops""#));
}

#[test]
fn test_template_literal() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // const msg = `Hello, ${name}!`
    let template = js.template_literal(vec!["Hello, ", "!"], vec![js.ident("name")]);
    let stmt = js.const_decl("msg", template);
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("const msg"));
    assert!(code.contains("`"));
    assert!(code.contains("Hello"));
    assert!(code.contains("name"));
}

#[test]
fn test_template_literal_multiple_expressions() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // const msg = `${greeting}, ${name}!`
    let template = js.template_literal(
        vec!["", ", ", "!"],
        vec![js.ident("greeting"), js.ident("name")],
    );
    let stmt = js.const_decl("msg", template);
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("const msg"));
    assert!(code.contains("greeting"));
    assert!(code.contains("name"));
}

#[test]
fn test_arrow_fn_with_block() {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // const fn = (x) => { return x * 2; }
    let arrow = js.arrow_fn_block(
        vec!["x"],
        vec![js.return_stmt(Some(js.binary(
            js.ident("x"),
            BinaryOperator::Multiplication,
            js.number(2.0),
        )))],
    );
    let stmt = js.const_decl("fn", arrow);
    let code = js.program(vec![stmt]).unwrap();

    assert!(code.contains("const fn"));
    assert!(code.contains("=>"));
    assert!(code.contains("return"));
    assert!(code.contains("x * 2"));
}

//! Example: Generating an API client with type-safe methods
//!
//! This demonstrates generating a client SDK with proper error handling.

use fob_gen::{Allocator, JsBuilder};
use oxc_ast::ast::Statement;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let allocator = Allocator::default();
    let js = JsBuilder::new(&allocator);

    // class ApiClient {
    //   async get(url) {
    //     const response = await fetch(url);
    //     if (!response.ok) {
    //       throw new Error(`HTTP error! status: ${response.status}`);
    //     }
    //     return await response.json();
    //   }
    // }

    // const response = await fetch(url)
    let fetch_call = js.call(js.ident("fetch"), vec![js.arg(js.ident("url"))]);
    let response_decl = js.const_decl("response", fetch_call);

    // if (!response.ok) { throw new Error(...) }
    let condition = js.not(js.member(js.ident("response"), "ok"));
    let error_msg = js.template_literal(
        vec!["HTTP error! status: ", ""],
        vec![js.member(js.ident("response"), "status")],
    );
    let throw_stmt = js.throw(js.new_expr(js.ident("Error"), vec![js.arg(error_msg)]));
    let if_stmt = js.if_stmt(condition, vec![throw_stmt], None);

    // return await response.json()
    let json_call = js.call(js.member(js.ident("response"), "json"), vec![]);
    let return_stmt = js.return_stmt(Some(json_call));

    // Build the get method
    let get_method = js.arrow_fn_block(vec!["url"], vec![response_decl, if_stmt, return_stmt]);

    // const ApiClient = { get: async (url) => {...} }
    let api_client_obj = js.object(vec![js.prop("get", get_method)]);
    let api_client_decl = js.const_decl("ApiClient", api_client_obj);

    // export default ApiClient
    let export_default = js.export_default(js.ident("ApiClient"));

    // Generate module
    let code = js.program(vec![api_client_decl, Statement::from(export_default)])?;

    println!("{}", code);
    Ok(())
}

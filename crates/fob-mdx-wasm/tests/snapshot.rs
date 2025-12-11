//! Snapshot tests for MDX compilation in WASM.

use fob_mdx_wasm::{WasmMdxOptions, compile_mdx};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!();

const PROPS_MDX: &str = include_str!("fixtures/props.mdx");

#[wasm_bindgen_test]
fn props_fixture_snapshot() {
    let options = WasmMdxOptions::new();
    let result = compile_mdx(PROPS_MDX, Some(options));

    assert!(result.is_ok(), "Compilation should succeed");

    let js_value = result.unwrap();
    let code = js_sys::Reflect::get(&js_value, &"code".into()).unwrap();
    let code_str = code.as_string().unwrap();

    let expected_code = r#"/*@jsxRuntime automatic @jsxImportSource react*/
function _createMdxContent(props) {
    return <div aria-label="some-label" data-value="value" {...rest}>
    Hello, world!
  </div>;
}
export default function MDXContent(props = {}) {
    const { wrapper: MDXLayout } = props.components || {};
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props} /></MDXLayout> : _createMdxContent(props);
}"#;

    assert_eq!(code_str, expected_code);
}

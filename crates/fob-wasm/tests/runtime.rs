#![cfg(target_arch = "wasm32")]

use joy_bundler_wasm::Fob;
use serde_json::json;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "./js/runtime.js")]
extern "C" {
    #[wasm_bindgen(js_name = bootstrap)]
    fn bootstrap(fs_root: Option<String>);
}

#[wasm_bindgen_test(async)]
async fn fails_without_entries_but_initializes_runtime() {
    bootstrap(None);
    let config = JsValue::from_serde(&json!({
        "entries": []
    }))
    .unwrap();

    let bundler = Fob::new(config).unwrap();
    let result = bundler.bundle().await;
    assert!(result.is_err());
}

#![cfg(target_arch = "wasm32")]

use eframe::wasm_bindgen::prelude::wasm_bindgen;
use matex_compiler::cas::{
    backend::{
        format::{NormalFormatter, ValueFormatter},
        runtime::Runtime,
        value::RuntimeVal,
    },
    frontend::{lexer::Lexer, parser::Parser},
};
use wasm_bindgen::JsValue;

use crate::app::app::MatexApp;

use super::abi::WasmRuntimeVal;

#[wasm_bindgen]
pub fn new_debug_app(canvas_id: String) {
    console_error_panic_hook::set_once();

    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async move {
        eframe::start_web(
            canvas_id.as_str(),
            web_options,
            Box::new(|_cc| Box::new(MatexApp::default())),
        )
        .await
        .expect("Failed to initialise eframe");
    });
}

#[wasm_bindgen]
pub fn run(source: String) -> JsValue {
    let mut parser = Parser::new(Lexer::new(&source).collect());
    match parser.parse() {
        Ok(program) => {
            let mut rt = Runtime::new();
            let value = rt.run(&program);
            return serde_wasm_bindgen::to_value(&value).unwrap();
        }
        Err(e) => serde_wasm_bindgen::to_value(&e).unwrap(),
    }
}

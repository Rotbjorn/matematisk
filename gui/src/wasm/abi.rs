use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct WasmRuntimeVal {
    typ: WasmRuntimeValueType,
}

pub enum WasmRuntimeValueType {
    Unit,
    Number,
    Symbol,
    Bool,
    Sum,
    Product,
    Exponent,
}

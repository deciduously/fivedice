// ffi.rs contains wrappers for various FFI uses

use wasm_bindgen::prelude::*;
use web_sys::Document;

// Alias for JsValue error
pub type Result<T> = std::result::Result<T, JsValue>;

/// Grab the document
pub fn get_document() -> Result<Document> {
    let window = web_sys::window().unwrap();
    Ok(window.document().unwrap())
}

/// use js Math.random() to get an integer in range [min, max)
pub fn js_gen_range(min: i64, max: i64) -> i64 {
    (js_sys::Math::floor(js_sys::Math::random() * (max as f64 - min as f64)) + min as f64) as i64
}

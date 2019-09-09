use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement};
/// Grab the body
pub fn get_body() -> HtmlElement {
        get_document()
                .body()
                .expect("Should locate <body> in document")
}

/// Grab the canvas
pub fn get_canvas() -> HtmlCanvasElement {
        get_body()
                .query_selector("canvas")
                .expect("Should find <canvas>")
                .unwrap()
                .dyn_into::<HtmlCanvasElement>()
                .expect("Should cast Element to HtmlCanvasElement")
}

/// Grab the context
pub fn get_context() -> CanvasRenderingContext2d {
        get_canvas()
                .get_context("2d")
                .expect("Should get rending context from <canvas>")
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .expect("Should cast to CanvasRenderingContext2d")
}

/// Grab the document
pub fn get_document() -> Document {
        get_window().document().expect("Should locate document")
}

/// Grab the window
fn get_window() -> web_sys::Window {
        web_sys::window().expect("Should locate window")
}

/// requestAnimationFrame
pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
        get_window()
                .request_animation_frame(f.as_ref().unchecked_ref())
                .expect("Should register `requestAnimationFrame`");
}

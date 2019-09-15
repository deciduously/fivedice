use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement};

// Attempt to return the page body
pub fn body() -> HtmlElement {
    get_body(document())
}

/// Attempt to locate what should be the only mounted <canvas> element
pub fn canvas() -> HtmlCanvasElement {
    get_canvas(body())
}

// Attempt to locate what should be the only CanvasRenderingContext2d
pub fn ctx() -> CanvasRenderingContext2d {
    get_context(canvas())
}

// Attempt to locate the document
pub fn document() -> Document {
    get_document(get_window())
}

/// Get body from document
pub fn get_body(document: Document) -> HtmlElement {
    document.body().expect("Should locate body")
}

/// Attempt to find canvas in body
pub fn get_canvas(body: HtmlElement) -> HtmlCanvasElement {
    body.query_selector("canvas")
        .expect("Should find <canvas>")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .expect("Should cast canvas to HtmlCanvasElement")
}

/// Get context from canvas
pub fn get_context(canvas: HtmlCanvasElement) -> CanvasRenderingContext2d {
    canvas
        .get_context("2d")
        .expect("Should get render context from canvas element")
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .expect("Should cast render context to CanvasRenderingContext2d")
}
/// Get document from window
pub fn get_document(window: web_sys::Window) -> Document {
    window.document().expect("Should locate document")
}

/// Grab the window
pub fn get_window() -> web_sys::Window {
    web_sys::window().expect("Should locate window")
}

/// requestAnimationFrame
pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    get_window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("Should register `requestAnimationFrame`");
}

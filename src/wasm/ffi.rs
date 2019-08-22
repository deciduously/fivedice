// ffi.rs contains all JS<->Rust interop

use crate::{
    draw::{WebSysCanvas, WindowEngine},
    game::Game,
};
use js_sys::Math::{floor, random};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement};

/// Grab the body
pub fn get_body() -> HtmlElement {
    get_document().body().expect("No <body> found in document")
}

/// Grab the canvas
pub fn get_canvas() -> HtmlCanvasElement {
    get_body()
        .query_selector("canvas")
        .expect("Could not find <canvas>")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .expect("Could not decipher canvas")
}

/// Grab the context
pub fn get_context() -> CanvasRenderingContext2d {
    get_canvas()
        .get_context("2d")
        .expect("Could not get rending context from <canvas>")
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .expect("Could not decipher rendering context")
}

/// Grab the document
pub fn get_document() -> Document {
    get_window()
        .document()
        .expect("No document found on window")
}

/// Grab the window
fn get_window() -> web_sys::Window {
    web_sys::window().expect("No window found")
}

/// requestAnimationFrame
pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    get_window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame`");
}

/// use js Math.random() to get an integer in range [min, max)
pub fn js_gen_range(min: i64, max: i64) -> i64 {
    (floor(random() * (max as f64 - min as f64)) + min as f64) as i64
}

/// Entry point for the module
#[allow(dead_code)]
#[wasm_bindgen(start)]
pub fn start() {
    // Instantiate canvas
    let renderable_context =
        Box::new(WebSysCanvas::new("Five Dice").expect("Could not instantiate canvas"));

    // Instantiate game
    let game = Box::new(Game::new());

    // Instantiate engine
    let engine = WindowEngine::new(renderable_context, game);

    // Run game
    engine.start();
}

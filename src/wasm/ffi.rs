// ffi.rs contains all JS<->Rust interop

use crate::{
    draw::{CanvasEngine, VALUES},
    game::Game,
};
use js_sys::Math::{floor, random};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
    console, CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement, MouseEvent, Window,
};

/// Grab the body
fn get_body() -> HtmlElement {
    get_document().body().expect("No <body> found in document")
}

/// Grab the canvas
fn get_canvas() -> HtmlCanvasElement {
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
fn get_document() -> Document {
    get_window()
        .document()
        .expect("No document found on window")
}

/// Grab the window
fn get_window() -> Window {
    web_sys::window().expect("No window found")
}

/// requestAnimationFrame
fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    get_window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame`");
}

/// use js Math.random() to get an integer in range [min, max)
pub fn js_gen_range(min: i64, max: i64) -> i64 {
    (floor(random() * (max as f64 - min as f64)) + min as f64) as i64
}

/// Entrypoint for the module
#[allow(dead_code)]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Set canvas dimensions
    let document = get_document();
    let body = get_body();
    // Mount the title and canvas elements
    append_text_element_attrs!(document, body, "h1", "FIVE DICE",);
    append_element_attrs!(document, body, "canvas",);

    get_canvas().set_width(VALUES.canvas_size.0);
    get_canvas().set_height(VALUES.canvas_size.1);

    // Instantiate game
    let game = Box::new(Game::new());

    // Instantiate Canvas engine
    let engine = Rc::new(RefCell::new(CanvasEngine::new(game)));

    // Add click listener
    // translate from page coords to canvas coords
    // shamelessly lifted from the RustWasm book but translated to Rust
    // https://rustwasm.github.io/book/game-of-life/interactivity.html
    {
        //let engine = engine.clone();
        let callback = Closure::wrap(Box::new(move |_evt: MouseEvent| {
            let canvas = get_canvas();
            let bounding_rect = canvas.get_bounding_client_rect();
            let scale_x = f64::from(canvas.width()) / bounding_rect.width();
            let scale_y = f64::from(canvas.height()) / bounding_rect.height();

            //let canvas_x = (f64::from(evt.client_x()) - bounding_rect.left()) * scale_x;
            //let canvas_y = (f64::from(evt.client_y()) - bounding_rect.top()) * scale_y;

            //TODO implement Clickable
            //engine.borrow_mut().handle_click(canvas_x, canvas_y);
            console::log_1(&format!("click at ({}, {})", scale_x, scale_y).into());
        }) as Box<dyn FnMut(_)>);

        get_canvas()
            .add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())
            .expect("Could not register event listener");
        callback.forget();
    }

    // Run the game loop
    // All iterations inside the loop can use the Rc.  Starts out empty
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        engine.borrow().draw();
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));
    // Kick off the loop
    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

// ffi.rs contains all JS<->Rust interfaces

use crate::{game::Game, CANVAS_X, CANVAS_Y};
use js_sys::Math::{floor, random};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Document, HtmlElement, Window};

/// Grab the body
pub fn get_body() -> HtmlElement {
    get_document().body().expect("No <body> found in document")
}

/// Grab the document
pub fn get_document() -> Document {
    get_window()
        .document()
        .expect("No document found on window")
}

/// Grab the window
pub fn get_window() -> Window {
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

/// Entrypoint for the module
#[allow(dead_code)]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Instantiate game object
    let game = Rc::new(RefCell::new(Game::new()));

    // Mount the canvas elements
    let document = get_document();
    let body = get_body();
    // Mount the title and canvas elements
    append_text_element_attrs!(document, body, "h1", "FIVE DICE",);
    append_element_attrs!(document, body, "canvas",);

    // Set up the height
    let canvas = document
        .query_selector("canvas")?
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    // Set canvas dimensions
    canvas.set_width(CANVAS_X);
    canvas.set_height(CANVAS_Y);

    // Add click listener
    // translate from page coords to canvas coords
    // shamelessly lifted from the RustWasm book but translated to Rust
    // https://rustwasm.github.io/book/game-of-life/interactivity.html
    {
        let game = game.clone();
        let callback = Closure::wrap(Box::new(move |evt: web_sys::MouseEvent| {
            let canvas = get_document()
                .query_selector("canvas")
                .expect("Could not find game screen")
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .expect("Could not find game canvas");
            let bounding_rect = canvas.get_bounding_client_rect();
            let scale_x = f64::from(canvas.width()) / bounding_rect.width();
            let scale_y = f64::from(canvas.height()) / bounding_rect.height();

            let canvas_x = (f64::from(evt.client_x()) - bounding_rect.left()) * scale_x;
            let canvas_y = (f64::from(evt.client_y()) - bounding_rect.top()) * scale_y;

            game.borrow_mut().handle_click(canvas_x, canvas_y);
        }) as Box<dyn FnMut(_)>);

        canvas.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())?;
        callback.forget();
    }

    // Run the game loop
    // All iterations inside the loop can use the Rc.  Starts out empty
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        game.borrow().draw().expect("Could not draw game");
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));
    // Kick off the loop
    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

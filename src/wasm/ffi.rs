// ffi.rs contains all JS<->Rust interop

use crate::{
    draw::{Color, Point, Region, Window, WindowEngine, WindowError, WindowResult, VALUES},
    game::Game,
};
use js_sys::Math::{floor, random};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
    console, CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement, MouseEvent,
};

/// Grab the body
fn get_body() -> HtmlElement {
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
fn get_document() -> Document {
    get_window()
        .document()
        .expect("No document found on window")
}

/// Grab the window
fn get_window() -> web_sys::Window {
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

/// Canvas implementation for WebSys
pub struct WebSysCanvas {
    ctx: Rc<CanvasRenderingContext2d>,
}

impl WebSysCanvas {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for WebSysCanvas {
    fn default() -> Self {
        Self {
            ctx: Rc::new(get_context()),
        }
    }
}

impl Window for WebSysCanvas {
    fn blank(&self) {
        self.ctx.clear_rect(
            0.0,
            0.0,
            VALUES.canvas_size.0.into(),
            VALUES.canvas_size.1.into(),
        )
    }
    fn rect(&self, region: Region) {
        self.ctx.rect(
            region.origin().x,
            region.origin().y,
            region.width(),
            region.height(),
        );
    }
    fn begin_path(&self) {
        self.ctx.begin_path();
    }
    fn draw_path(&self) {
        self.ctx.stroke();
    }
    fn set_color(&self, color: Color) {
        self.ctx.set_stroke_style(&format!("{}", color).into());
    }
    fn text(&self, text: &str, font: &str, origin: Point) -> WindowResult<()> {
        self.ctx.set_font(font);
        if self.ctx.fill_text(text, origin.x, origin.y).is_err() {
            return Err(WindowError::Text);
        }
        Ok(())
    }
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
    append_element_attrs!(
        document,
        body,
        "canvas",
        ("width", &format!("{}", VALUES.canvas_size.0)),
        ("height", &format!("{}", VALUES.canvas_size.1))
    );

    // Instantiate game engine
    let renderable_context = Box::new(WebSysCanvas::new());

    // Instantiate game
    let game = Box::new(Game::new());

    // Instantiate Canvas engine
    let engine = Rc::new(RefCell::new(WindowEngine::new(renderable_context, game)));

    // Add click listener
    // translate from page coords to canvas coords
    // shamelessly lifted from the RustWasm book but translated to Rust
    // https://rustwasm.github.io/book/game-of-life/interactivity.html
    {
        //let engine = engine.clone();
        let callback = Closure::wrap(Box::new(move |evt: MouseEvent| {
            let canvas = get_canvas();
            let bounding_rect = canvas.get_bounding_client_rect();
            let scale_x = f64::from(canvas.width()) / bounding_rect.width();
            let scale_y = f64::from(canvas.height()) / bounding_rect.height();

            let canvas_x = (f64::from(evt.client_x()) - bounding_rect.left()) * scale_x;
            let canvas_y = (f64::from(evt.client_y()) - bounding_rect.top()) * scale_y;

            //TODO implement Clickable
            //engine.borrow_mut().handle_click(canvas_x, canvas_y);
            let _click: Point = (canvas_x, canvas_y).into();
        }) as Box<dyn FnMut(_)>);

        get_canvas()
            .add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())
            .expect("Could not register event listener");
        callback.forget();

        // Run the game loop
        // All iterations inside the loop can use the Rc.  Starts out empty
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            if let Err(e) = engine.borrow().draw() {
                console::log_2(&"Error: ".into(), &e.into());
            };
            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));
        // Kick off the loop
        request_animation_frame(g.borrow().as_ref().unwrap());

        Ok(())
    }
}

use crate::{
    error::{Result, WindowError},
    ffi::{body, canvas, ctx, document, request_animation_frame},
    traits::Widget,
    types::{Color, Point, Region, Values},
};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{console, CanvasRenderingContext2d, MouseEvent};

/// Trait representing a canvas to be drawn to.  For now, only supports CanvasRenderingContext2d
pub trait Window {
    /// Blank the window
    fn blank(&self);
    // Get the constant values for this window
    fn get_values(&self) -> Values;
    /// Draw a rectangle
    fn rect(&self, region: Region, color: Color);
    /// Begin/rest a path - should we let the engine handle this??
    /// its more efficient to batch calls, so for now I'm letting the user decide when to do that
    // TODO Eventually a DSL will let batches happen
    fn begin_path(&self);
    /// Draw the current path
    fn draw_path(&self);
    /// Set pen color
    fn set_color(&self, color_str: Color);
    /// Draw some text
    fn text(&self, text: &str, font: &str, origin: Point) -> Result<()>;
    /// Get the width of the text
    fn text_width(&self, text: &str) -> Result<f64>;
}

/// Alias for a reference-counted pointer to a Window object
pub type WindowPtr = Rc<Box<dyn Window>>;

/// Canvas implementation for WebSys
pub struct WebSysCanvas {
    ctx: CanvasRenderingContext2d,
    values: Values,
}

impl WebSysCanvas {
    pub fn new(title: &str) -> Result<Self> {
        console_error_panic_hook::set_once();
        // set up Values
        let values = Values::default();
        // Set up page
        let document = document();
        let body = body();
        // Mount the title and canvas elements
        append_text_element_attrs!(document, body, "h1", title,);
        append_element_attrs!(
            document,
            body,
            "canvas",
            ("width", &format!("{}", values.canvas_region.width())),
            ("height", &format!("{}", values.canvas_region.height()))
        );
        // Add click listener
        // translate from page coords to canvas coords
        // https://rustwasm.github.io/book/game-of-life/interactivity.html but in Rust, not JS
        let callback = Closure::wrap(Box::new(move |evt: MouseEvent| {
            let canvas = canvas();
            let bounding_rect = canvas.get_bounding_client_rect();
            let scale_x = f64::from(canvas.width()) / bounding_rect.width();
            let scale_y = f64::from(canvas.height()) / bounding_rect.height();

            let canvas_x = (f64::from(evt.client_x()) - bounding_rect.left()) * scale_x;
            let canvas_y = (f64::from(evt.client_y()) - bounding_rect.top()) * scale_y;

            let click: Point = (canvas_x, canvas_y).into();
            CLICKS.with(|cs| cs.borrow_mut().push_back(click));
        }) as Box<dyn FnMut(_)>);
        canvas()
            .add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())
            .expect("Should register event listener");
        callback.forget();
        Ok(Self::default())
    }
}

impl Default for WebSysCanvas {
    fn default() -> Self {
        Self {
            ctx: ctx(),
            values: Values::default(),
        }
    }
}

impl Window for WebSysCanvas {
    fn blank(&self) {
        let r = self.get_values().canvas_region;
        self.ctx.clear_rect(0.0, 0.0, r.width(), r.height());
    }
    fn get_values(&self) -> Values {
        self.values
    }
    fn rect(&self, region: Region, color: Color) {
        self.set_color(color);
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
    fn text(&self, text: &str, font: &str, origin: Point) -> Result<()> {
        self.ctx.set_font(font);
        if self.ctx.fill_text(text, origin.x, origin.y).is_err() {
            return Err(WindowError::Text);
        }
        Ok(())
    }
    fn text_width(&self, text: &str) -> Result<f64> {
        let res = self.ctx.measure_text(text);
        match res {
            Ok(tm) => Ok(tm.width()),
            Err(e) => Err(WindowError::JsVal(e)),
        }
    }
}

// Static holder for clicks
// TODO - probably doesn't need to be a queue
// The odds of a user registering multiple distinct clicks in a frame are next to none
// Especially because it just gets drained to a Vec for processing anyway
thread_local! {
    static CLICKS: RefCell<VecDeque<Point>> = RefCell::new(VecDeque::new());
}

//trait ValuesTrait {}

/// Top-level canvas engine object
pub struct WindowEngine<T: 'static> {
    window: WindowPtr,
    element: Box<dyn Widget<MSG = T>>,
}

impl<T> WindowEngine<T> {
    pub fn new(w: Box<dyn Window>, element: Box<dyn Widget<MSG = T>>) -> Self {
        Self {
            window: Rc::new(w),
            element,
        }
    }

    /// Draw elements
    /// Takes a list of clicks to resolve first
    pub fn draw(&mut self, clicks: Vec<Point>) -> Result<()> {
        // handle any received clicks
        for click in clicks {
            self.element
                .handle_click(Point::default(), click, Rc::clone(&self.window))?;
        }
        // clear canvas
        self.window.blank();
        // Draw element
        let w = Rc::clone(&self.window);
        if let Err(e) = self.element.mount_widget(Point::default()).draw(w, true) {
            console::error_2(&"Draw".into(), &format!("{}", e).into());
        };
        Ok(())
    }

    /// Start engine
    pub fn start(self) {
        let engine = Rc::new(RefCell::new(self));
        // Run the game loop
        // Initiate animation_frame() callback
        // All iterations inside the loop can use the Rc.  Starts out empty
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            // pass any clicks out of the queue into the engine
            let mut rcvd_clicks: Vec<Point> = Vec::new();
            CLICKS.with(|cs| {
                let len = cs.borrow().len();
                for _ in 0..len {
                    match cs.borrow_mut().pop_front() {
                        Some(c) => rcvd_clicks.push(c),
                        None => break,
                    }
                }
            });
            if let Err(e) = engine.borrow_mut().draw(rcvd_clicks) {
                console::error_2(&"Draw error".into(), &format!("{}", e).into());
            }
            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));
        // Kick off the loop
        request_animation_frame(g.borrow().as_ref().unwrap());
    }
}

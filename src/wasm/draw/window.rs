use super::{ffi::*, *};
use web_sys::{CanvasRenderingContext2d, MouseEvent};
/// Window error type
#[derive(Debug)]
pub enum WindowError {
    Element,
    JsVal(JsValue),
    OutOfBounds,
    Text,
}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Element => write!(f, "Could not append element to DOM"),
            Self::JsVal(js) => write!(f, "{:#?}", js),
            Self::Text => write!(f, "Could not add text to the window"),
            Self::OutOfBounds => write!(f, "Attempted to scroll cursor out of bounds!"),
        }
    }
}

impl From<JsValue> for WindowError {
    fn from(other: JsValue) -> Self {
        WindowError::JsVal(other)
    }
}

impl Into<JsValue> for WindowError {
    fn into(self) -> JsValue {
        format!("{}", self).into()
    }
}

impl std::error::Error for WindowError {}

pub type WindowResult<T> = std::result::Result<T, WindowError>;

/// Trait representing a canvas to be drawn to.  For now, only supports CanvasRenderingContext2d
pub trait Window {
    /// Blank the window
    fn blank(&self);
    /// Draw a rectangle
    fn rect(&self, region: Region);
    /// Begin/rest a path - should we let the engine handle this??
    /// its more efficient to batch calls, so for now I'm letting the user decide when to do that
    // TODO Eventually a DSL will let batches happen
    fn begin_path(&self);
    /// Draw the current path
    fn draw_path(&self);
    /// Set pen color
    fn set_color(&self, color_str: Color);
    /// Draw some text
    fn text(&self, text: &str, font: &str, origin: Point) -> WindowResult<()>;
}

/// Alias for a reference-counted pointer to a Window object
pub type WindowPtr = Rc<Box<dyn Window>>;

/// Canvas implementation for WebSys
pub struct WebSysCanvas {
    ctx: Rc<CanvasRenderingContext2d>,
}

impl WebSysCanvas {
    pub fn new(title: &str) -> WindowResult<Self> {
        // Set up page
        let document = get_document();
        let body = get_body();
        // Mount the title and canvas elements
        append_text_element_attrs!(document, body, "h1", title,);
        append_element_attrs!(
            document,
            body,
            "canvas",
            ("width", &format!("{}", VALUES.canvas_size.0)),
            ("height", &format!("{}", VALUES.canvas_size.1))
        );

        Ok(Self::default())
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

/// Top-level canvas engine object
/// // TODO maybe a good spot to store values?
pub struct WindowEngine {
    window: WindowPtr,
    element: MountedWidget,
}

impl WindowEngine {
    pub fn new(w: Box<dyn Window>, e: Box<dyn Widget>) -> Self {
        let window = Rc::new(w);
        let mounted_widget = e.mount_widget(Point::default());
        Self {
            window,
            element: mounted_widget,
        }
    }

    /// Draw elements
    pub fn draw(&self) -> Result<()> {
        // set canvas dimensions
        get_canvas().set_width(VALUES.canvas_size.0);
        get_canvas().set_height(VALUES.canvas_size.1);
        // clear canvas
        self.window.blank();
        // Draw element
        let w = Rc::clone(&self.window);
        self.element.draw(w)?;
        Ok(())
    }

    /// Start engine
    pub fn start(self) {
        let engine = Rc::new(RefCell::new(self));
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
        }
        {
            // Run the game loop
            // All iterations inside the loop can use the Rc.  Starts out empty
            let f = Rc::new(RefCell::new(None));
            let g = f.clone();
            *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                engine
                    .borrow()
                    .draw()
                    .unwrap_or_else(|_| panic!("Window error on draw"));
                request_animation_frame(f.borrow().as_ref().unwrap());
            }) as Box<dyn FnMut()>));
            // Kick off the loop
            request_animation_frame(g.borrow().as_ref().unwrap());
        }
    }
}

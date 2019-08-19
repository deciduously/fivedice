// draw.rs contains the Drawable/Clickable traits and canvas rendering engine, as well as generic widgets like Button
use crate::{context::Context, error::*, ffi::get_canvas};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

// You somehow need each thing to know where it is
// You need a better abstraction over the canvas.

// Have a Canvas with a draw() and handle_click() method, not on Game
// It will hold Drawables.  Each Drawable should be able to hold its own drawables, but then pass back up
// so the parent object can continue drawing where it left off.

// TODO look into AsRef()/AsMut()?

/// A single coordinate point on the canvas, top left is 0,0
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl From<(f64, f64)> for Point {
    fn from(other: (f64, f64)) -> Self {
        Self {
            x: other.0,
            y: other.1,
        }
    }
}

/// A rectangular region on the canvas
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Region {
    origin: Point,
    width: f64,
    height: f64,
}

impl From<(Point, f64, f64)> for Region {
    fn from(bits: (Point, f64, f64)) -> Self {
        Self {
            origin: bits.0,
            width: bits.1,
            height: bits.2,
        }
    }
}

impl From<(f64, f64, f64, f64)> for Region {
    fn from(bits: (f64, f64, f64, f64)) -> Self {
        ((bits.0, bits.1).into(), bits.2, bits.3).into()
    }
}

/// Trait representing things that can be drawn to the canvas
/// Each frame the whole tree is re-mounted
/// It will have child Drawable widgets
pub trait Drawable {
    /// Draw this game element with the given top left corner
    /// Only ever called once mounted.  Returns the bottom right corner of what was painted
    fn draw_at(
        &self,
        top_left: Point,
        context: &Context,
        ctx: &CanvasRenderingContext2d,
    ) -> Result<Point>;
    /// Return the origin, width, and height of this element
    fn get_region(&self) -> Region;
    /// Return all drawables this Drawable contains
    fn get_widgets(&self) -> Vec<Box<dyn Drawable>>;
    /// Mount this Drawable with a given position
    fn mount(&self, top_left: Point) -> Vec<Box<MountedDrawable>>;
}

/// Trait representing Drawables that can be clicked
pub trait Clickable {
    // Handle a click at the given coordinates
    // No-op if coordinates outside of this boundary
    // If inside, execute f
    fn handle_click(&self, click: Point, c: dyn FnMut());
}

/// Wrapper struct for a Drawable that has been mounted to the canvas
struct MountedDrawable
{
    cursor: Point,
    region: Region,
    drawable: Box<dyn Drawable>,
}

impl MountedDrawable
{
    fn new(drawable: Box<dyn Drawable>, top_left: Point) -> Self {
        Self {
            cursor: drawable.get_region().origin,
            drawable,
            // TODO how to get width/height?
            region: (top_left, 10.0, 10.0).into(),
        }
    }

    /// Draw this element and update the cursor
    fn draw(&self, context: &Context, ctx: &CanvasRenderingContext2d) {
        // Draw all constituent wdigets, updating the cursor after each
        self.cursor = self.draw_at(self.region.origin, context, ctx).unwrap();
    }

    /// Return the next drawing position
    fn get_cursor_pos(&self) -> Point {
        self.cursor
    }
}

impl Drawable for MountedDrawable {
        fn draw_at(
        &self,
        top_left: Point,
        context: &Context,
        ctx: &CanvasRenderingContext2d,
    ) -> Result<Point> {
        self.drawable.draw_at(top_left, context, ctx)
    }
    fn get_region(&self) -> Region {
        self.drawable.get_region()
    }

    fn mount(&self, top_left: Point) -> Vec<Box<MountedDrawable>> {

    }
}

/// Top-level canvas engine object
pub struct CanvasEngine {
    canvas: Box<HtmlCanvasElement>,
    elements: Vec<MountedDrawable>,
}

impl CanvasEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a reference to the rendering context
    pub fn get_context(&self) -> Result<&CanvasRenderingContext2d> {
        let ctx = self.canvas.get_context("2d");

        if let Err(_) = ctx {
            Err(FiveDiceError::Interop(
                "Could not get render context".into(),
            ))
        } else {
            let ret = ctx.unwrap().unwrap().dyn_into::<CanvasRenderingContext2d>();
            if let Err(_) = ret {
                Err(FiveDiceError::Interop(
                    "Could not find canvas context".into(),
                ))
            } else {
                Ok(&ret.unwrap())
            }
        }
    }

    /// Mount widget
    pub fn mount(&mut self, w: Box<dyn Drawable>) {
        // you've got to mount all the elements
        // somehow go through the widgets recursively
        // so each widget needs to return its children
        // with absolute positions
        
        // Mount the drawable and push it
        self.elements.push(MountedDrawable::new(w, self.get_cursor_pos()));
    }

    /// Get the next cursor position
    // TODO add padding
    fn get_cursor_pos(&self) -> Point {
        // last widget's bottrom right.  X to 0 (or values.padding), Y to that dot + padding
        let last_drawn_region = self.elements[self.elements.len() - 1].get_region();
        (0.0, last_drawn_region.origin.y + last_drawn_region.height).into()
    }
}

impl Default for CanvasEngine {
    fn default() -> Self {
        Self {
            canvas: Box::new(get_canvas()),
            elements: Vec::new(),
        }
    }
}

//
// HELPER FUNCTIONS
// These will disappear once I get the Canvas going
//

/// Draw a button - should be a struct!!
pub fn draw_button(
    text: &str,
    top_left: Point,
    context: &Context,
    ctx: &web_sys::CanvasRenderingContext2d,
) -> Result<()> {
    let values = context.values;
    ctx.begin_path();

    // Configure font
    ctx.set_font(&values.get_font_string());

    // Configure button size
    let text_width = ctx.measure_text(text).unwrap().width();
    let button_width = text_width + values.padding;
    let button_height = values.get_button_height();

    // Set color
    ctx.set_stroke_style(&JsValue::from_str(values.button_color));
    // Stage border
    ctx.rect(top_left.x, top_left.y, button_width, button_height);
    // Stage button text
    if let Err(_) = ctx.fill_text(
        text,
        top_left.x + (values.padding / 2.0),
        top_left.y + (button_height / 2.0),
    ) {
        return Err(FiveDiceError::Canvas("button".into()));
    };

    // Draw and return
    ctx.stroke();
    Ok(())
}

/// Draw some text -= also a struct, impl Drawable!
pub fn draw_text(
    text: &str,
    top_left: Point,
    ctx: &web_sys::CanvasRenderingContext2d,
) -> Result<()> {
    ctx.begin_path();
    if let Err(_) = ctx.fill_text(text, top_left.x, top_left.y) {
        return Err(FiveDiceError::Canvas("button".into()));
    };
    ctx.stroke();
    Ok(())
}

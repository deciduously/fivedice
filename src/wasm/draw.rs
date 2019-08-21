// draw.rs contains the Drawable/Clickable traits and canvas rendering engine, as well as generic widgets like Button
use crate::{
    error::*,
    ffi::{get_canvas, get_context},
};
use std::{cell::Cell, fmt};
use wasm_bindgen::JsValue;
use web_sys::{console, CanvasRenderingContext2d};

// You somehow need each thing to know where it is
// You need a better abstraction over the canvas.

// Have a Canvas with a draw() and handle_click() method, not on Game
// It will hold Drawables.  Each Drawable should be able to hold its own drawables, but then pass back up
// so the parent object can continue drawing where it left off.

// TODO look into AsRef()/AsMut()?

// TODO docs
// There are two distinct things: Drawables and Widgets.  I'm calling it a Mounted right now, its really a Widget
// Widgets contain drawables
// To implement Widget, you need to define how many rows of elements you want, and how many elements are in each row
// what's a good way to do this?
// Each row is rendered horizontally
// rows are rendered one after another
// The padding between rows and spaces is defined in Values - which should live on CanvasEngine, I now realize
// Each row will have the width of the largest element rendered

/// A single coordinate point on the canvas, top left is 0,0
#[derive(Debug, Default, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    fn new() -> Self {
        Self::default()
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// (x, y), from (0, 0) at top left
impl From<(f64, f64)> for Point {
    fn from(other: (f64, f64)) -> Self {
        Self {
            x: other.0,
            y: other.1,
        }
    }
}

impl Into<JsValue> for Point {
    fn into(self) -> JsValue {
        format!("{}", self).into()
    }
}

/// A rectangular region on the canvas
#[derive(Debug, Default, Clone, Copy)]
pub struct Region {
    origin: Point,
    width: f64,
    height: f64,
}

/// (top_left, bottom_right)
impl From<(Point, Point)> for Region {
    fn from(bits: (Point, Point)) -> Self {
        Self {
            origin: bits.0,
            width: bits.1.x - bits.0.x,
            height: bits.1.y - bits.0.y,
        }
    }
}

/// (origin, width, height)
impl From<(Point, f64, f64)> for Region {
    fn from(bits: (Point, f64, f64)) -> Self {
        Self {
            origin: bits.0,
            width: bits.1,
            height: bits.2,
        }
    }
}

/// (origin_x, origin_y, width, height)
impl From<(f64, f64, f64, f64)> for Region {
    fn from(bits: (f64, f64, f64, f64)) -> Self {
        ((bits.0, bits.1).into(), bits.2, bits.3).into()
    }
}

/// Trait representing things that can be drawn to the canvas
pub trait Drawable {
    /// Draw this game element with the given top left corner
    /// Only ever called once mounted.  Returns the bottom right corner of what was painted
    fn draw_at(&self, top_left: Point, ctx: &CanvasRenderingContext2d) -> Result<Point>;
    /// Get the Region of the bounding box of this drawable
    fn get_region(&self, top_left: Point) -> Region;
}

/// Trait representing sets of 0 or more Drawables
/// Each one can have variable number rows and elements in each row
pub trait Widget {
    /// Make this object into a Widget
    // TODO make a DSL for this - right now they're all:
    // {
    //     let ret p MountedWidget::new(top_left);
    //     //push some elements
    //     ret
    // }
    fn mount_widget(&self, top_left: Point) -> MountedWidget;
}

/// Trait representing Drawables that can be clicked
pub trait Clickable: Drawable {
    // Handle a click at the given coordinates
    // No-op if coordinates outside of this boundary
    // If inside, execute f
    fn handle_click(&self, click: Point, c: dyn FnMut());
}

/// A container struct for a widget
#[derive(Default)]
pub struct MountedWidget {
    children: Vec<Vec<Box<dyn Widget>>>,
    drawable: Option<Box<dyn Drawable>>,
    cursor: Cell<Point>,
    top_left: Point,
}

impl MountedWidget {
    pub fn new(top_left: Point) -> Self {
        Self {
            children: vec![vec![]],
            drawable: None,
            cursor: Cell::new(top_left),
            top_left,
        }
    }

    /// Draw this element and update the cursor
    fn draw(&self, ctx: &CanvasRenderingContext2d) -> Result<Point> {
        // Draw all constituent widgets, updating the cursor after each
        // Draw any child widgets
        if self.children.len() > 0 {
            for row in &self.children {
                if row.len() > 0 {
                    for child in row {
                        // mount the child
                        let mounted_child = child.mount_widget(self.cursor.get());
                        // draw the child
                        self.cursor.set(mounted_child.draw(ctx)?);
                        // advance the cursor horizontally by padding and back to where we started vertically

                        self.scroll_horizontal(VALUES.padding)?;
                        self.scroll_vertical(-(self.cursor.get().y - self.top_left.y))?;
                    }
                    // advance the cursor back to the beginning of the next line down
                    self.scroll_vertical(VALUES.padding)?;
                    self.scroll_horizontal(-(self.cursor.get().x - VALUES.padding))?;
                }
            }
        }
        // draw self, if present
        if let Some(d) = &self.drawable {
            self.cursor.set(d.draw_at(self.cursor.get(), ctx)?);
        }
        Ok(self.cursor.get())
    }

    // TODO maybe these should be one function with a parameter?
    /// Add a new element to the current row
    pub fn push_current_row(&mut self, d: Box<dyn Widget>) {
        let num_rows = self.children.len();
        let idx = if num_rows > 0 { num_rows - 1 } else { 0 };
        self.children[idx].push(d);
    }

    /// Add a new element to a new row
    pub fn push_new_row(&mut self, d: Box<dyn Widget>) {
        self.children.push(vec![d]);
    }

    /// Scroll cursor horizontally
    pub fn scroll_horizontal(&self, offset: f64) -> Result<()> {
        let current = self.cursor.get();
        let new_point = (current.x + offset, current.y).into();
        if !VALUES.fits_canvas(new_point) {
            return Err(FiveDiceError::OutOfBounds);
        }
        self.cursor.set(new_point);
        Ok(())
    }

    /// Scroll cursor vertical
    pub fn scroll_vertical(&self, offset: f64) -> Result<()> {
        let current = self.cursor.get();
        let new_point = (current.x, current.y + offset).into();
        if !VALUES.fits_canvas(new_point) {
            return Err(FiveDiceError::OutOfBounds);
        }
        self.cursor.set(new_point);
        Ok(())
    }

    /// Set drawable for this widget - overrides any currently set
    pub fn set_drawable(&mut self, d: Box<dyn Drawable>) {
        self.drawable = Some(d);
    }
}

impl fmt::Display for MountedWidget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Mounted Widget at {} with {} rows of children,{} drawable",
            self.top_left,
            self.children.len(),
            if self.drawable.is_some() { "" } else { " not" }
        )
    }
}

impl Drawable for MountedWidget {
    fn draw_at(&self, _: Point, ctx: &CanvasRenderingContext2d) -> Result<Point> {
        // Return new cursor position, leaving at bottom right

        // TODO THIS IS OVERRIDING THE ACTUAL DRAW_AT
        Ok(self.draw(ctx)?)
    }
    fn get_region(&self, _: Point) -> Region {
        (self.top_left, self.cursor.get()).into()
    }
}

//
// Reusable Drawables
//

// TODO Text

// TODO Button

// Values configuration

/// Layout values
#[derive(Debug, Clone, Copy)]
pub struct Values {
    /// Total size of canvas (width, height)
    pub canvas_size: (u32, u32),
    /// Size of one die square
    pub die_dimension: f64,
    /// Padding value used all over the place
    pub padding: f64,
    /// What the roll dice button says
    pub reroll_button_text: &'static str,
    /// What color to use for button border
    pub button_color: &'static str,
    /// What font to use for buttons
    pub button_font: &'static str,
    /// What size font on buttons
    pub button_font_size: u8,
}

impl Values {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return whether the given point fits on this canvas size
    fn fits_canvas(&self, p: Point) -> bool {
        (p.x as u32) < self.canvas_size.0
            && p.x >= 0.0
            && (p.y as u32) < self.canvas_size.1
            && p.y >= 0.0
    }

    /// Put the font size and the font together
    pub fn get_font_string(&self) -> String {
        format!("{}px {}", self.button_font_size, self.button_font)
    }
}

impl Default for Values {
    fn default() -> Self {
        Self {
            canvas_size: (800, 600),
            die_dimension: 50.0,
            padding: 10.0,
            reroll_button_text: "Roll!",
            button_color: "black",
            button_font: "Arial",
            button_font_size: 16,
        }
    }
}

lazy_static! {
    /// Instantiate static values object
    pub static ref VALUES: Values = Values::new();
}

// TODO Parameterize canvas drawing
// After you get a successful paint!
// Instead of passing the CanvasRenderingContext2d, make dedicated draw_line(), draw_circle(), draw_text(), and that will handle the details

/// Top-level canvas engine object
pub struct CanvasEngine {
    ctx: CanvasRenderingContext2d,
    element: Option<MountedWidget>,
}

impl CanvasEngine {
    pub fn new(w: Box<dyn Widget>) -> Self {
        let mounted_widget = w.mount_widget(Point::default());
        console::log_2(
            &"Mounting to canvas: ".into(),
            &format!("{}", mounted_widget).into(),
        );
        Self {
            ctx: get_context(),
            element: Some(mounted_widget),
        }
    }

    /// Draw elements
    pub fn draw(&self) -> Result<()> {
        // set canvas dimensions
        get_canvas().set_width(VALUES.canvas_size.0);
        get_canvas().set_height(VALUES.canvas_size.1);
        // clear canvas
        self.ctx.clear_rect(
            0.0,
            0.0,
            VALUES.canvas_size.0.into(),
            VALUES.canvas_size.1.into(),
        );
        // Draw element, if any
        if let Some(w) = &self.element {
            w.draw(&self.ctx)?;
        }
        Ok(())
    }
}

/*
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
*/

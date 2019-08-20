// draw.rs contains the Drawable/Clickable traits and canvas rendering engine, as well as generic widgets like Button
use crate::{error::*, ffi::get_context};
use web_sys::CanvasRenderingContext2d;

// You somehow need each thing to know where it is
// You need a better abstraction over the canvas.

// Have a Canvas with a draw() and handle_click() method, not on Game
// It will hold Drawables.  Each Drawable should be able to hold its own drawables, but then pass back up
// so the parent object can continue drawing where it left off.

// TODO look into AsRef()/AsMut()?

// TODO
// There are two distinct thinggs - drawables and Widgets.  I'm calling it a Mounted right now, its really a Widget
// Widgets contain drawables
// To implement Widget, you need to define how many rows of elements you want, and how many elements are in each row
// what's a good way to do this?
// Each row is rendered horizontally
// rows are rendered one after another
// The padding between rows and spaces is defined in Values - which should live on CanvasEngine, I now realize
// Each row will have the width of the largest element rendered

/// A single coordinate point on the canvas, top left is 0,0
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    fn new() -> Self {
        Self::default()
    }
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
#[derive(Debug, Default, Clone, Copy, PartialEq)]
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
pub trait Drawable {
    /// Draw this game element with the given top left corner
    /// Only ever called once mounted.  Returns the bottom right corner of what was painted
    fn draw_at(
        &self,
        top_left: Point,
        ctx: &CanvasRenderingContext2d,
    ) -> Result<Point>;
    /// Get the Region of the bounding box of this drawable
    fn get_region(&self, top_left: Point) -> Region;
}

/// Trait representing sets of 0 or more Drawables
/// Each one can have variable number rows and elements in each row
pub trait Widget {
    /// Make this object into a Widget
    /// TODO make a DSL for this - right now they're all:
    /// {
    ///     let ret p MountedWidget::new(top_left);
    ///     //push some elements
    ///     ret
    /// }
    fn make_widget(self, top_left: Point) -> MountedWidget;
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
    cursor: Point,
    top_left: Point,
}

impl MountedWidget {
    pub fn new(top_left: Point) -> Self {
        Self {
            children: vec![vec![]],
            cursor: top_left,
            top_left,
        }
    }

    /// Draw this element and update the cursor
    fn draw(&self, ctx: &CanvasRenderingContext2d) {
        // Draw all constituent widgets, updating the cursor after each
        self.cursor = self
            .draw_at(self.get_region(self.cursor).origin, ctx)
            .unwrap();
    }

    /// Return the next drawing position
    fn get_cursor_pos(&self) -> Point {
        self.cursor
    }

    // TODO maybe these should be one function with a parameter?
    /// Add a new element to the current row
    pub fn push_current_row(&mut self, d: Box<dyn Drawable>) {
        unimplemented!()
    }
    /// Add a new element to a new row
    pub fn push_new_row(&mut self, d: Box<dyn Drawable>) {
        unimplemented!()
    }
}

impl Drawable for MountedWidget {
    fn draw_at(
        &self,
        top_left: Point,
        ctx: &CanvasRenderingContext2d,
    ) -> Result<Point> {
        unimplemented!()
        // iterate through the rows of the drawables vec
        // each row should be rendered horizontally
        // then start a new line until out of rows
    }
    fn get_region(&self, top_left: Point) -> Region {
        unimplemented!()
        // you've got to add up the regions of all the contained drawables
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

    /// Get button height
    pub fn get_button_height(&self) -> f64 {
        f64::from(self.button_font_size) + (self.padding + 2.0)
    }

    /// Put the font size and the font together
    pub fn get_font_string(&self) -> String {
        format!("{}px {}", self.button_font_size, self.button_font)
    }

    /// Get top left corner of hand display
    pub fn dice_origin(&self) -> (f64, f64) {
        (self.padding, 0.0)
    }
    /*
    /// Get the top left corner of the reroll dice button (topleft, bottomright), both as (x, y)
    /// // TODO remove - this will end up with Clickable on this specific Button object
    pub fn reroll_button_corners(
        &self,
        context: &CanvasRenderingContext2d,
    ) -> ((f64, f64), (f64, f64)) {
        let text_width = context
            .measure_text(self.reroll_button_text)
            .unwrap()
            .width();
        let button_width = text_width + self.padding;
        let button_height = self.get_button_height();
        let top_left = (
            self.padding,
            self.dice_origin().1 + self.die_dimension + (self.padding * 2.0),
        );
        let bottom_right = (top_left.0 + button_width, top_left.1 + button_height);
        (top_left, bottom_right)
    }

    /// Get the corners of the start over button
    /// TODO make it easier to get the corners from just one corner and the text
    pub fn start_over_button_corners(
        &self,
        context: &CanvasRenderingContext2d,
    ) -> ((f64, f64), (f64, f64)) {
        let text_width = context.measure_text("Start Over").unwrap().width();
        let top_left = (
            self.padding,
            (self.reroll_button_corners(context).1).1 + self.padding,
        );
        let bottom_right = (
            top_left.0 + text_width + self.padding,
            top_left.1 + self.get_button_height(),
        );
        (top_left, bottom_right)
    }
    */
}

impl Default for Values {
    fn default() -> Self {
        Self {
            canvas_size: (640, 480),
            die_dimension: 50.0,
            padding: 10.0,
            reroll_button_text: "Roll!",
            button_color: "black",
            button_font: "Arial",
            button_font_size: 16,
        }
    }
}

/// Instantiate static values object
pub static VALUES: Values = Values::new();

/// Top-level canvas engine object
pub struct CanvasEngine {
    ctx: CanvasRenderingContext2d,
    cursor: Point,
    elements: Vec<MountedWidget>,
}

impl CanvasEngine {
    pub fn new() -> Self {
        Self::default()
    }
    /// Mount widget
    pub fn mount(&mut self, w: Box<dyn Widget>) {
        // you've got to mount all the elements
        // somehow go through the widgets recursively
        // so each widget needs to return its children
        // with absolute positions
        // Mount the drawable and push it
        self.elements.push(w.make_widget(self.get_cursor_pos()));
    }

    /// Draw elements
    pub fn draw(&self) {
        unimplemented!()
    }

    /// Get the next cursor position
    // TODO add padding
    fn get_cursor_pos(&self) -> Point {
        // last widget's bottom right.  X to 0 (or values.padding), Y to that dot + padding
        let last_drawn_region =
            self.elements[self.elements.len() - 1].get_region(self.cursor);
        (0.0, last_drawn_region.origin.y + last_drawn_region.height).into()
    }
}

impl Default for CanvasEngine {
    fn default() -> Self {
        Self {
            ctx: get_context(),
            cursor: Point::new(),
            elements: Vec::new(),
        }
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

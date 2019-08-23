#[macro_use]
extern crate lazy_static;

use std::{cell::RefCell, cmp::Ordering, fmt, ops::AddAssign, rc::Rc, str::FromStr};
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::console;
/// DOM manipulation macros
#[macro_use]
mod dom;
/// FFI initiation
mod ffi;
/// Window and WebSysCanvas
pub mod window;

use ffi::get_context;
use window::*;

/// Trait representing things that can be drawn to the canvas
pub trait Drawable {
    /// Draw this game element with the given top left corner
    /// Only ever called once mounted.  Returns the bottom right corner of what was painted
    fn draw_at(&self, top_left: Point, ctx: WindowPtr) -> WindowResult<Point>;
    /// Get the Region of the bounding box of this drawable
    // TODO maybe should get ctx as well, for measure_text?  to avoid extra get_context() call
    fn get_region(&self, top_left: Point) -> Region;
}

/// Trait representing sets of 0 or more Drawables
/// Each one can have variable number rows and elements in each row
pub trait Widget {
    /// Get the total of all regions of this widget
    fn get_region(&self, top_left: Point) -> Region;
    /// Make this object into a Widget
    // TODO make a DSL for this - right now they're all:
    // {
    //     let ret p MountedWidget::new(top_left);
    //     //push some elements
    //     ret
    // }
    fn mount_widget(&self) -> MountedWidget;
}

/// Trait representing Drawables that can be clicked
pub trait Clickable: Drawable {
    // Handle a click at the given coordinates
    // No-op if coordinates outside of this boundary
    // If inside, execute f
    fn handle_click(&self, click: Point, c: dyn FnMut());
}

/// A single coordinate point on the canvas, top left is 0,0
#[derive(Debug, Default, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Set to point, specifically on canvas
    fn set_to(&mut self, p: Point) -> WindowResult<()> {
        if !VALUES.fits_canvas(p) {
            return Err(WindowError::OutOfBounds(*self, p));
        } else {
            self.x = p.x;
            self.y = p.y;
            Ok(())
        }
    }
    /// Horizontal offset
    fn horiz_offset(&mut self, offset: f64) -> WindowResult<()> {
        self.set_to((self.x + offset, self.y).into())?;
        Ok(())
    }
    /// Vertical offset
    fn vert_offset(&mut self, offset: f64) -> WindowResult<()> {
        self.set_to((self.x, self.y + offset).into())?;
        Ok(())
    }
    /// Return the distance between self and other
    fn distance(&self, other: Point) -> f64 {
        return ((other.x - self.x).powi(2) + (other.y - self.y).powi(2)).sqrt();
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Return which is closer to the origin
        if self.distance(Point::default()) < other.distance(Point::default()) {
            Some(Ordering::Less)
        } else if self.x == other.x && self.y == other.y {
            Some(Ordering::Equal)
        } else {
            Some(Ordering::Greater)
        }
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
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
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Region {
    o: Point,
    w: f64,
    h: f64,
}

impl Region {
    /// Return this region's bottom right
    pub fn bottom_right(&self) -> Point {
        (self.o.x + self.w, self.o.y + self.h).into()
    }
    /// Return this region's top right
    pub fn top_right(&self) -> Point {
        (self.o.x + self.w, self.o.y).into()
    }
    /// Getter for origin
    pub fn origin(&self) -> Point {
        self.o
    }
    /// Getter for height
    pub fn height(&self) -> f64 {
        self.h
    }
    /// Getter for width
    pub fn width(&self) -> f64 {
        self.w
    }
}

impl AddAssign for Region {
    /// # Examples
    /// ```
    /// extern crate widget_grid;
    /// use widget_grid::Region;
    ///
    /// # fn main() {
    ///     let mut orig: Region = (0.0, 0.0, 10.0, 10.0).into();
    ///     let other1: Region = (4.0, 6.0, 20.0, 8.0).into();
    ///     orig += other1;
    ///     assert_eq!(orig, (0.0, 0.0, 24.0, 14.0).into());
    /// # }
    /// # fn five_dice() {
    ///     let dice = vec![
    ///        (10.0, 0.0, 50.0, 50.0).into(),
    ///        (70.0, 0.0, 50.0, 50.0).into(),
    ///        (130.0, 0.0, 50.0, 50.0).into(),
    ///        (190.0, 0.0, 50.0, 50.0).into(),
    ///        (250.0, 0.0, 50.0, 50.0).into(),
    ///    ];
    ///    let mut total: Region = dice[0];
    ///    for e in dice.iter().skip(1) {
    ///        total += *e;
    ///    }
    ///    let expected = (10.0, 0.0, 300.0, 50.0).into();
    ///    assert_eq!(total, expected);
    /// # }
    /// ```
    fn add_assign(&mut self, other: Region) {
        // keep the top_leftiest bottom_rightiest bottom_right
        let bottom_right = self.bottom_right();
        let o_bottom_right = other.bottom_right();
        let top_right = self.top_right();
        let o_top_right = other.top_right();
        let o_winner = if self.o < other.origin() {
            self.o
        } else {
            other.origin()
        };
        *self = Self {
            o: o_winner,
            w: if top_right.x > o_top_right.x {
                top_right.x
            } else {
                o_top_right.x
            },
            h: if bottom_right.y > o_bottom_right.y {
                bottom_right.y
            } else {
                o_bottom_right.y
            },
        };
    }
}

/// (top_left, bottom_right)
impl From<(Point, Point)> for Region {
    fn from(bits: (Point, Point)) -> Self {
        Self {
            o: bits.0,
            w: bits.1.x - bits.0.x,
            h: bits.1.y - bits.0.y,
        }
    }
}

/// (origin, width, height)
impl From<(Point, f64, f64)> for Region {
    fn from(bits: (Point, f64, f64)) -> Self {
        Self {
            o: bits.0,
            w: bits.1,
            h: bits.2,
        }
    }
}

/// (origin_x, origin_y, width, height)
impl From<(f64, f64, f64, f64)> for Region {
    fn from(bits: (f64, f64, f64, f64)) -> Self {
        ((bits.0, bits.1).into(), bits.2, bits.3).into()
    }
}

/// Color type, RGB
#[derive(Debug, Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.r == 255 {
            write!(f, "red")
        } else if self.r == 0 && self.g == 0 && self.b == 0 {
            write!(f, "black")
        } else {
            write!(f, "#{:x}{:x}{:x}", self.r, self.g, self.b)
        }
    }
}

impl FromStr for Color {
    type Err = WindowError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "black" => Ok(Color::new(0, 0, 0)),
            "red" => Ok(Color::new(255, 0, 0)),
            _ => unimplemented!(),
        }
    }
}

/// A container struct for a widget
#[derive(Default)]
pub struct MountedWidget {
    children: Vec<Vec<Box<dyn Widget>>>,
    drawable: Option<Box<dyn Drawable>>,
}

impl MountedWidget {
    pub fn new() -> Self {
        Self {
            children: vec![vec![]],
            drawable: None,
        }
    }

    /*
    THIS WORKED
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
    */

    /// Draw this element and update the cursor
    pub fn draw(&self, top_left: Point, ctx: WindowPtr) -> WindowResult<Point> {
        // Draw all constituent widgets, updating the cursor after each
        // Draw any child widgets
        let mut cursor = top_left;
        if !&self.children.is_empty() {
            for row in &self.children {
                if !row.is_empty() {
                    // store the largest vertical offset in the row and row top left
                    //let mut vertical_offset = 0.0;
                    //let row_top_left = self.cursor.get();
                    // Draw each child
                    for child in row {
                        // Mount the child
                        console::log_2(&"Mounting child at".into(), &format!("{}", cursor).into());
                        let mounted_child = child.mount_widget();
                        // draw the child
                        let ctx = Rc::clone(&ctx);
                        let end_point = mounted_child.draw(cursor, ctx)?;
                        //`// check if tallest
                        //`let offset = end_point.y - row_top_left.y;
                        //`if offset > vertical_offset {
                        //`    vertical_offset = offset;
                        //`}
                        // scroll to the next top_left
                        cursor.horiz_offset(VALUES.padding + end_point.x)?;
                    }
                }
                // advance the cursor back to the beginning of the next line down
                // TODO die_dimension is a stand, in, use the vert_offset stuff
                cursor.vert_offset(VALUES.padding + VALUES.die_dimension + VALUES.padding)?;
                cursor.horiz_offset(-(cursor.x - VALUES.padding))?;
            }
        }
        // draw self, if present
        if let Some(d) = &self.drawable {
            cursor.set_to(d.draw_at(cursor, ctx)?)?;
        }
        Ok(cursor)
    }
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

    /// Set drawable for this widget - overrides any currently set
    pub fn set_drawable(&mut self, d: Box<dyn Drawable>) {
        self.drawable = Some(d);
    }
}

impl fmt::Display for MountedWidget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Mounted Widget: {} rows of children,{} drawable",
            self.children.len(),
            if self.drawable.is_some() { "" } else { " not" }
        )
    }
}

impl Drawable for MountedWidget {
    fn draw_at(&self, top_left: Point, ctx: WindowPtr) -> WindowResult<Point> {
        // Return new cursor position, leaving at bottom right
        Ok(self.draw(top_left, ctx)?)
    }
    fn get_region(&self, top_left: Point) -> Region {
        // Add up all the regions.
        let mut ret = (top_left, 0.0, 0.0).into();
        let mut cursor = top_left;
        for row in &self.children {
            for child in row {
                let child_top_left = cursor;
                let r = child.get_region(child_top_left);
                ret += r;
                cursor.horiz_offset(VALUES.padding).expect("Illegal scroll");
            }
        }
        // add any drawable
        if let Some(d) = &self.drawable {
            let new_region = d.get_region(cursor);
            ret += d.get_region(new_region.origin());
        }
        ret
    }
}

//
// Reusable Drawables
//

/// A widget that just draws some text
pub struct Text {
    text: String,
}

impl Text {
    pub fn new(s: &str) -> Self {
        Self { text: s.into() }
    }
}

impl Drawable for Text {
    fn draw_at(&self, top_left: Point, ctx: WindowPtr) -> WindowResult<Point> {
        ctx.begin_path();
        ctx.text(&self.text, &VALUES.get_font_string(), top_left)?;
        ctx.draw_path();
        Ok(Drawable::get_region(self, top_left).bottom_right())
    }

    fn get_region(&self, top_left: Point) -> Region {
        // TODO remove this get_context()?
        let text_size = get_context()
            .measure_text(&self.text)
            .expect("Could not measure text");
        (top_left, text_size.width(), f64::from(VALUES.font_size)).into()
    }
}

impl Widget for Text {
    fn mount_widget(&self) -> MountedWidget {
        let mut ret = MountedWidget::new();
        ret.set_drawable(Box::new(Text::new(&self.text)));
        ret
    }
    fn get_region(&self, top_left: Point) -> Region {
        Drawable::get_region(self, top_left)
    }
}

/// A clickable widget
// pub struct Button {}

// Values configuration
// TODO this is very tightly coupled
// Last thing before you can split out

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
    pub font: &'static str,
    /// What size font on buttons
    pub button_font_size: u8,
    /// General font size
    pub font_size: u8,
}

impl Values {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return whether the given point fits on this canvas size
    fn fits_canvas(&self, p: Point) -> bool {
        p.x <= f64::from(self.canvas_size.0)
            && p.x >= 0.0
            && p.y <= f64::from(self.canvas_size.1)
            && p.y >= 0.0
    }

    /// Put the font size and the font together
    pub fn get_font_string(&self) -> String {
        format!("{}px {}", self.font_size, self.font)
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
            font: "Arial",
            button_font_size: 16,
            font_size: 12,
        }
    }
}

lazy_static! {
    /// Instantiate static values object
    pub static ref VALUES: Values = Values::new();
}

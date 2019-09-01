#[macro_use]
extern crate lazy_static;

use std::{cmp::Ordering, convert::AsRef, fmt, ops::AddAssign, rc::Rc, str::FromStr};
use wasm_bindgen::{prelude::*, JsCast, JsValue};
// use web_sys::console;
/// DOM manipulation macros
#[macro_use]
mod dom;
/// Error type
pub mod error;
/// FFI initiation
mod ffi;
/// Window and WebSysCanvas
pub mod window;

pub use error::*;
pub use window::*;

// TODO
// Ideally, I don't want to expose Drawable, just widget
// Also, I don't want to have to implement Widget manually
// You should be able to define a 2D vector of Box<dyn Widget>
// and pass it to a function, that builds the MountedWidget for you
// it should also be able to auto-derive get_region(), that's a solved problem

// TODO Builder Pattern all the things - widget, text, drawable

/// Trait representing things that can be drawn to the canvas
pub trait Drawable {
    /// Draw this game element with the given top left corner
    /// Only ever called once mounted.  Returns the bottom right corner of what was painted
    fn draw_at(&self, top_left: Point, w: WindowPtr) -> Result<Point>;
    /// Get the Region of the bounding box of this drawable
    fn get_region(&self, top_left: Point, w: WindowPtr) -> Result<Region>;
}

/// Trait representing sets of 0 or more Drawables
/// Each one can have variable number rows and elements in each row
pub trait Widget<T> {
    /// Just a bool signalling if this item is clickable
    /// Get the total of all regions of this widget
    // TODO i'd like to not have to have this in Widget
    fn get_region(&self, top_left: Point, w: WindowPtr) -> Result<Region>;
    /// Handle a click in this region
    fn handle_click(&mut self, top_left: Point, click: Point, w: WindowPtr) -> Result<Option<T>>;
    /// Make this object into a Widget.  Takes an optional callback
    // TODO make a DSL for this - right now they're all:
    // {
    //     let ret p MountedWidget::new(top_left);
    //     //push some elements
    //     ret
    // }
    fn mount_widget(&self) -> MountedWidget<T>;
}

/// Callback type
/// /// thanks to https://github.com/yewstack/yew/blob/master/src/callback.rs
pub struct Callback<T> {
    f: Rc<dyn Fn() -> T>,
}

impl<T> Callback<T> {
    /// Call this callback
    pub fn call(&self) -> T {
        (self.f)()
    }
}

impl<T> Clone for Callback<T> {
    fn clone(&self) -> Self {
        Self {
            f: Rc::clone(&self.f),
        }
    }
}

impl<T> PartialEq for Callback<T> {
    fn eq(&self, other: &Callback<T>) -> bool {
        Rc::ptr_eq(&self.f, &other.f)
    }
}

impl<T, F: Fn() -> T + 'static> From<F> for Callback<T> {
    fn from(func: F) -> Self {
        Self { f: Rc::new(func) }
    }
}

/// A single coordinate point on the canvas, top left is 0,0
#[derive(Debug, Default, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Set to point, specifically on canvas
    pub fn set_to(&mut self, p: Point) -> Result<()> {
        if !VALUES.fits_canvas(p) {
            Err(WindowError::OutOfBounds(*self, p))
        } else {
            self.x = p.x;
            self.y = p.y;
            Ok(())
        }
    }
    /// Horizontal offset
    pub fn horiz_offset(&mut self, offset: f64) -> Result<()> {
        self.set_to((self.x + offset, self.y).into())?;
        Ok(())
    }
    /// Vertical offset
    pub fn vert_offset(&mut self, offset: f64) -> Result<()> {
        self.set_to((self.x, self.y + offset).into())?;
        Ok(())
    }
    /// Return the distance between self and other
    fn distance(&self, other: Point) -> f64 {
        ((other.x - self.x).powi(2) + (other.y - self.y).powi(2)).sqrt()
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Return which is closer to the origin
        if self.distance(Point::default()) < other.distance(Point::default()) {
            Some(Ordering::Less)
        } else if (self.x - other.x).abs() < std::f64::EPSILON
            && (self.y - other.y).abs() < std::f64::EPSILON
        {
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
    /// Check if given point is within this region
    pub fn contains(&self, p: Point) -> bool {
        self.o.x <= p.x
            && self.o.y <= p.y
            && self.bottom_right().x >= p.x
            && self.bottom_right().y >= p.y
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

    /// Add/remove width
    pub fn width_offset(&mut self, offset: f64) {
        self.w += offset;
    }
    /// Add/remove height
    pub fn height_offset(&mut self, offset: f64) {
        self.h += offset;
    }
}

impl AddAssign for Region {
    /// # Examples
    /// ```
    /// extern crate widget_grid;
    /// use widget_grid::Region;
    ///
    /// # fn main() {
    ///     // origin in region, larger
    ///     let mut orig: Region = (0.0, 0.0, 10.0, 10.0).into();
    ///     let other1: Region = (4.0, 6.0, 20.0, 8.0).into();
    ///     orig += other1;
    ///     assert_eq!(orig, (0.0, 0.0, 24.0, 14.0).into());
    ///     // five dice
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
    ///    // self to self
    ///    let mut r1: Region = (0.0, 0.0, 10.0, 10.0).into();
    ///    let r2 = (0.0, 0.0, 10.0, 10.0).into();
    ///    r1 += r2;
    ///    assert_eq!(r1, r2);
    ///    // orig encompasses operand
    ///    let r1: Region = (0.0, 0.0, 10.0, 10.0).into();
    ///    let mut r2 = r1;
    ///    r2 += (2.0, 2.0, 2.0, 2.0).into();
    ///    assert_eq!(r1, r2);
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
pub struct MountedWidget<T> {
    children: Vec<Vec<Box<dyn Widget<T>>>>,
    drawable: Option<Box<dyn Drawable>>,
}

impl<T> MountedWidget<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Draw this element - pass true to actually render elements, false to just return the bottom right
    pub fn draw(&self, top_left: Point, w: WindowPtr) -> Result<Point> {
        // Draw all constituent widgets, updating the cursor after each
        // Draw any child widgets
        let mut cursor = top_left;
        let mut bottom_right = top_left;
        let mut vertical_offset = 0.0;
        for row in &self.children {
            let row_top_left = cursor;
            // Draw each child
            for child in row {
                // Mount the child
                let child_top_left = cursor;
                let mounted_child = child.mount_widget();
                // draw the child
                cursor.set_to(mounted_child.draw(child_top_left, Rc::clone(&w))?)?;
                // check if tallest
                let offset = cursor.y - row_top_left.y;
                if offset > vertical_offset {
                    vertical_offset = offset;
                }
                // store possible bottom right
                let child_bottom_right = mounted_child
                    .get_region(child_top_left, Rc::clone(&w))?
                    .bottom_right();
                if child_bottom_right > bottom_right {
                    bottom_right = child_bottom_right;
                }
                cursor.vert_offset(-(cursor.y - child_top_left.y))?;
                cursor.horiz_offset(VALUES.padding)?;
            }
            // advance the cursor back to the beginning of the next line down
            cursor.vert_offset((VALUES.padding * 2.0) + vertical_offset)?;
            cursor.horiz_offset(-(cursor.x - VALUES.padding))?;
        }
        // draw self, if present
        if let Some(d) = &self.drawable {
            // The drawable should start at the top left!!!
            // a widgets drawable should encompass all child elements
            // widget.drawable.get_region().origin() <= widget.get_get_region.origin() &&
            // widget.drawable.get_region().bottom_right >= last_child.get_region().bottom_right()
            cursor.set_to(d.draw_at(top_left, w)?)?;
            bottom_right = cursor;
        }
        // Return bottom right
        Ok(bottom_right)
    }
    /// Add a new element to the current row
    pub fn push_current_row(&mut self, d: Box<dyn Widget<T>>) {
        let num_rows = self.children.len();
        let idx = if num_rows > 0 { num_rows - 1 } else { 0 };
        self.children[idx].push(d);
    }

    /// Add a new element to a new row
    pub fn push_new_row(&mut self, d: Box<dyn Widget<T>>) {
        self.children.push(vec![d]);
    }

    /// Set drawable for this widget - overrides any currently set
    pub fn set_drawable(&mut self, d: Box<dyn Drawable>) {
        self.drawable = Some(d);
    }

    /// Get the entire region encompassing this MountedWidget
    pub fn get_region(&self, top_left: Point, w: WindowPtr) -> Result<Region> {
        // TODO this is the same as drawing but...doesn't draw, and i'm gonna use it again for handle-click!
        if let Some(d) = &self.drawable {
            d.get_region(top_left, w)
        } else {
            let mut cursor = top_left;
            let mut bottom_right = top_left;
            for row in &self.children {
                for child in row {
                    let child_top_left = cursor;
                    let region = child.get_region(child_top_left, Rc::clone(&w))?;
                    if region.bottom_right() > bottom_right {
                        bottom_right = region.bottom_right();
                    }
                    cursor.vert_offset(-(cursor.y - child_top_left.y))?;
                    cursor.horiz_offset(VALUES.padding)?;
                }
            }
            Ok((top_left, bottom_right).into())
        }
    }

    /// Handle a click
    pub fn click(&mut self, top_left: Point, click: Point, w: WindowPtr) -> Result<Option<T>> {
        // iterate through widgets, handle all their clicks, handle drawable's click
        let mut cursor = top_left;
        for (rowi, _) in self.children.iter().enumerate() {
            for (childi, _) in self.children[rowi].iter().enumerate() {
                let child_top_left = cursor;
                match self.children[rowi][childi].mount_widget().click(
                    child_top_left,
                    click,
                    Rc::clone(&w),
                )? {
                    Some(m) => return Ok(Some(m)), // if a hit returns, that's it - pass it on up
                    None => {}
                }

                cursor.vert_offset(-(cursor.y - child_top_left.y))?;
                cursor.horiz_offset(VALUES.padding)?;
            }
            cursor.vert_offset(VALUES.padding + VALUES.die_dimension + VALUES.padding)?;
            cursor.horiz_offset(-(cursor.x - VALUES.padding))?;
        }
        Ok(None)
    }
}

impl<T> Default for MountedWidget<T> {
    fn default() -> Self {
        Self {
            children: vec![vec![]],
            drawable: None,
        }
    }
}

impl<T> fmt::Display for MountedWidget<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Mounted Widget: {} rows of children,{} drawable",
            self.children.len(),
            if self.drawable.is_some() { "" } else { " not" }
        )
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
    fn draw_at(&self, top_left: Point, w: WindowPtr) -> Result<Point> {
        w.begin_path();
        w.text(&self.text, &VALUES.get_font_string(), top_left)?;
        w.draw_path();
        Ok(Drawable::get_region(self, top_left, w)?.bottom_right())
    }

    fn get_region(&self, top_left: Point, w: WindowPtr) -> Result<Region> {
        Ok((
            top_left,
            w.text_width(&self.text)?,
            f64::from(VALUES.font_size),
        )
            .into())
    }
}

impl<T> Widget<T> for Text {
    fn mount_widget(&self) -> MountedWidget<T> {
        let mut ret = MountedWidget::new();
        // TODO see if the Cow helps with this?
        ret.set_drawable(Box::new(Text::new(&self.text)));
        ret
    }
    fn handle_click(&mut self, _: Point, _: Point, _: WindowPtr) -> Result<Option<T>> {
        Ok(None)
    }
    fn get_region(&self, top_left: Point, w: WindowPtr) -> Result<Region> {
        Drawable::get_region(self, top_left, w)
    }
}

/// Generic button type.  Optionally takes a "bottom right" point as a width and height
/// Takes a callback to call upon click and a value to pass to the callback
#[derive(Clone)]
pub struct Button<T> {
    bottom_right: Option<Point>,
    callback: Option<Callback<T>>,
    text: String,
}

impl<T> Button<T>
where
    T: 'static,
{
    pub fn new(s: &str, bottom_right: Option<Point>, callback: Option<Callback<T>>) -> Self {
        Self {
            bottom_right,
            callback,
            text: s.into(),
        }
    }
}

impl<T> Drawable for Button<T> {
    fn draw_at(&self, top_left: Point, w: WindowPtr) -> Result<Point> {
        w.begin_path();
        let outline = Drawable::get_region(self, top_left, Rc::clone(&w))?;
        w.rect(outline);
        w.text(
            &self.text,
            &VALUES.get_font_string(),
            (
                top_left.x + (VALUES.padding / 2.0),
                top_left.y + (VALUES.padding * 2.0),
            )
                .into(),
        )?;
        w.draw_path();
        Ok(outline.bottom_right())
    }

    fn get_region(&self, top_left: Point, w: WindowPtr) -> Result<Region> {
        match self.bottom_right {
            Some(p) => Ok((top_left, p.x, p.y).into()),
            None => Ok((
                top_left,
                w.text_width(&self.text)? + VALUES.padding,
                f64::from(VALUES.font_size) + VALUES.padding * 2.0,
            )
                .into()),
        }
    }
}

impl<T: 'static> Widget<T> for Button<T> {
    fn get_region(&self, top_left: Point, ctx: WindowPtr) -> Result<Region> {
        Drawable::get_region(self, top_left, ctx)
    }
    fn handle_click(&mut self, top_left: Point, click: Point, w: WindowPtr) -> Result<Option<T>> {
        if Drawable::get_region(self, top_left, w)?.contains(click) {
            match &self.callback {
                Some(f) => Ok(Some(f.call())),
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }
    fn mount_widget(&self) -> MountedWidget<T> {
        let mut ret = MountedWidget::new();
        // TODO why can't you use the derived Clone??
        let self_clone = Button::new(&self.text, self.bottom_right, self.callback.clone());
        ret.set_drawable(Box::new(self_clone));
        ret
    }
}

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

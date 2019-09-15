use crate::error::{Result, WindowError};
use std::{cmp::Ordering, fmt, ops::AddAssign, rc::Rc, str::FromStr};
use wasm_bindgen::JsValue;

/// Callback type
// thanks to https://github.com/yewstack/yew/blob/master/src/callback.rs with some differences
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
            "blue" => Ok(Color::new(0, 0, 255)),
            "green" => Ok(Color::new(0, 255, 0)),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum FontStyle {
    Arial,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self::Arial
    }
}

/// A font style and size
#[derive(Debug, Clone, Copy)]
pub struct Font {
    size: u8,
    style: FontStyle,
}

impl Font {
    // Getter for font size as float
    pub fn height(self) -> f64 {
        f64::from(self.size)
    }
}

impl Default for Font {
    fn default() -> Self {
        Self {
            size: 16,
            style: FontStyle::default(),
        }
    }
}

impl fmt::Display for Font {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}px {:?}", self.size, self.style)
    }
}

/// A single coordinate point on the canvas, top left is 0,0
#[derive(Debug, Default, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Set to new point
    pub fn set_to(&mut self, p: Point) -> Result<()> {
        self.x = p.x;
        self.y = p.y;
        Ok(())
    }
    /// Horizontal offset
    pub fn horiz_offset(&mut self, offset: f64) -> Result<()> {
        self.set_to((self.x + offset, self.y).into())
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
    /// use widget_grid::types::Region;
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

// Values configuration

/// Layout values
#[derive(Debug, Clone, Copy)]
pub struct Values {
    /// Total size of canvas (width, height)
    pub canvas_region: Region,
    /// Padding value between widgets
    pub padding: f64,
}

impl Values {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Values {
    fn default() -> Self {
        Self {
            canvas_region: (0.0, 0.0, 800.0, 600.0).into(),
            padding: 10.0,
        }
    }
}

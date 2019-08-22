// draw.rs contains the Drawable/Clickable traits and canvas rendering engine, as well as generic widgets like Button
use crate::{
    error::*,
    ffi::{get_body, get_canvas, get_context, get_document, request_animation_frame},
};
use std::{
    cell::{Cell, RefCell},
    cmp::Ordering,
    fmt,
    ops::AddAssign,
    rc::Rc,
    str::FromStr,
};
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, MouseEvent};

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

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.x < other.x && self.y < other.y {
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
    /// # fn main() {
    ///     let mut orig: Region = (0.0, 0.0, 10.0, 10.0).into();
    ///     let other1: Region = (4.0, 6.0, 20.0, 8.0).into();
    ///     orig += other1;
    ///     assert_eq!(orig, (0.0, 0.0, 16.0, 14.0));
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
            write!(f, "#{}{}{}", self.r, self.g, self.b)
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
    fn draw(&self, ctx: WindowPtr) -> WindowResult<Point> {
        // Draw all constituent widgets, updating the cursor after each
        // Draw any child widgets
        if !self.children.is_empty() {
            for row in &self.children {
                if !row.is_empty() {
                    // first mount all children in row
                    let mounted_children = row.iter().fold(vec![], |mut acc, c| {
                        // mount the child
                        let mounted_child = c.mount_widget(self.cursor.get());
                        // advance cursor
                        self.cursor
                            .set(mounted_child.get_region(self.cursor.get()).bottom_right());
                        acc.push(mounted_child);
                        acc
                    });
                    // Then draw them
                    self.cursor.set(self.top_left);
                    for (i, _) in row.iter().enumerate() {
                        // draw the child
                        let ctx = Rc::clone(&ctx);
                        self.cursor.set(mounted_children[i].draw(ctx)?);
                        // advance the cursor horizontally by padding and back to where we started vertically
                        self.scroll_horizontal(VALUES.padding)?;
                        self.scroll_vertical(-(self.cursor.get().y - self.top_left.y))?;
                    }
                    // advance the cursor back to the beginning of the next line down
                    // RIGHT HERE it needs to be the mounted child's region
                    // find tallest
                    let vertical_offset = mounted_children
                        .iter()
                        .map(|c| {
                            let ret = c.get_region(self.cursor.get());
                            self.cursor.set(ret.bottom_right());
                            ret.height() as u32
                        })
                        .max()
                        .unwrap_or(self.cursor.get().y as u32);
                    self.scroll_vertical(VALUES.padding + f64::from(vertical_offset))?;
                    self.cursor
                        .set((VALUES.padding, self.cursor.get().y).into());
                }
            }
        }
        // draw self, if present
        if let Some(d) = &self.drawable {
            self.cursor.set(d.draw_at(self.cursor.get(), ctx)?);
        }
        Ok(self.cursor.get())
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

    /// Scroll cursor horizontally
    pub fn scroll_horizontal(&self, offset: f64) -> WindowResult<()> {
        let current = self.cursor.get();
        let new_point = (current.x + offset, current.y).into();
        if !VALUES.fits_canvas(new_point) {
            return Err(WindowError::OutOfBounds);
        }
        self.cursor.set(new_point);
        Ok(())
    }

    /// Scroll cursor vertical
    pub fn scroll_vertical(&self, offset: f64) -> WindowResult<()> {
        let current = self.cursor.get();
        let new_point = (current.x, current.y + offset).into();
        if !VALUES.fits_canvas(new_point) {
            return Err(WindowError::OutOfBounds);
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
    fn draw_at(&self, _: Point, ctx: WindowPtr) -> WindowResult<Point> {
        // Return new cursor position, leaving at bottom right

        Ok(self.draw(ctx)?)
    }
    fn get_region(&self, _: Point) -> Region {
        // Add up all the regions.
        let mut ret = (self.top_left, 0.0, 0.0).into();
        for row in &self.children {
            for child in row {
                let res = child.mount_widget(self.cursor.get());
                self.cursor
                    .set(res.get_region(self.cursor.get()).bottom_right());
                ret += res.get_region(self.top_left);
            }
        }
        // add any drawable
        if let Some(d) = &self.drawable {
            let new_region = d.get_region(self.cursor.get());
            //self.cursor.set(new_region.bottom_right());
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
        Ok(self.get_region(top_left).bottom_right())
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
    fn mount_widget(&self, top_left: Point) -> MountedWidget {
        let mut ret = MountedWidget::new(top_left);
        ret.set_drawable(Box::new(Text::new(&self.text)));
        ret
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
    pub font_size: u8,
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
*/

#[cfg(test)]
mod test {
    #[test]
    fn test_add_adding_region() {
        use super::Region;
        let mut orig: Region = (0.0, 0.0, 10.0, 10.0).into();
        let other: Region = (4.0, 6.0, 20.0, 8.0).into();
        orig += other;
        assert_eq!(orig, (0.0, 0.0, 24.0, 14.0).into());
    }
    #[test]
    fn test_add_adding_up() {
        use super::Region;
        let mut orig: Region = (10.0, 10.0, 5.0, 5.0).into();
        let other: Region = (4.0, 16.0, 10.0, 10.0).into();
        orig += other;
        assert_eq!(orig, (4.0, 16.0, 15.0, 26.0).into());
    }

    #[test]
    fn test_add_disparate() {
        use super::Region;
        let mut orig: Region = (0.0, 0.0, 5.0, 5.0).into();
        let other: Region = (13.0, 10.0, 5.0, 5.0).into();
        orig += other;
        assert_eq!(orig, (0.0, 0.0, 18.0, 15.0).into());
    }
}

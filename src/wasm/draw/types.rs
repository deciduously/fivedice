use super::{ffi::get_context, *};
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
    pub fn draw(&self, ctx: WindowPtr) -> WindowResult<Point> {
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

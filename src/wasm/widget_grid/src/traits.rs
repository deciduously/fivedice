use crate::{
    window::WindowPtr,
    {fmt, Point, Region, Result, VALUES},
};
use std::rc::Rc;
// TODO
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
pub trait Widget {
    type MSG;
    /// Get the total of all regions of this widget
    fn get_region(&self, top_left: Point, w: WindowPtr) -> Result<Region>;
    /// Handle a click in this region
    fn handle_click(
        &mut self,
        top_left: Point,
        click: Point,
        w: WindowPtr,
    ) -> Result<Option<Self::MSG>>;
    /// Make this object into a Widget.  Takes an optional callback
    // TODO make a DSL for this - right now they're all:
    // {
    //     let ret p MountedWidget::new(top_left);
    //     //push some elements
    //     ret
    // }
    fn mount_widget(&self) -> MountedWidget<Self::MSG>;
}

/// A container struct for a widget
pub struct MountedWidget<T> {
    children: Vec<Vec<Box<dyn Widget<MSG = T>>>>,
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
            // a widget's drawable should encompass all child elements
            // widget.drawable.get_region().origin() <= widget.get_get_region.origin() &&
            // widget.drawable.get_region().bottom_right >= last_child.get_region().bottom_right()
            cursor.set_to(d.draw_at(top_left, w)?)?;
            bottom_right = cursor;
        }
        // Return bottom right
        Ok(bottom_right)
    }
    /// Add a new element to the current row
    pub fn push_current_row(&mut self, d: Box<dyn Widget<MSG = T>>) {
        let num_rows = self.children.len();
        let idx = if num_rows > 0 { num_rows - 1 } else { 0 };
        self.children[idx].push(d);
    }

    /// Add a new element to a new row
    pub fn push_new_row(&mut self, d: Box<dyn Widget<MSG = T>>) {
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

    /// Handle a click - broken??? - 0 die is correct, 1 die hits die 4, rest dead. seems like cursor increments too much?
    pub fn click(&mut self, top_left: Point, click: Point, w: WindowPtr) -> Result<Option<T>> {
        // iterate through widgets, handle all their clicks, h/andle drawable's click
        let mut cursor = top_left;
        for row in self.children.iter_mut() {
            for child in row.iter_mut() {
                let child_top_left = cursor;
                // if you change this to child.mount_widget().click() it all breaks (and probably shouldn't)
                if let Some(m) = child.handle_click(child_top_left, click, Rc::clone(&w))? {
                    return Ok(Some(m)); // if a hit returns, that's it - pass it on up
                }
                // advance cursor to next child
                // set to bottom right first
                cursor.set_to(
                    child
                        .get_region(child_top_left, Rc::clone(&w))?
                        .bottom_right(),
                )?;
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

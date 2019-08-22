use super::*;
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

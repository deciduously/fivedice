#[macro_use]
extern crate lazy_static;

use std::{cmp::Ordering, convert::AsRef, fmt, ops::AddAssign, rc::Rc, str::FromStr};
use types::{Color, Point, Region};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};

/// DOM manipulation macros
#[macro_use]
mod dom;
/// Error type
pub mod error;
/// FFI initiation
pub mod ffi;
/// Drawable and Widget traits, as well as MountedWidget type
pub mod traits;
/// Various type definitions
pub mod types;
/// Resuable widgets
pub mod widgets;
/// Window and WebSysCanvas
pub mod window;

// Re-export error
pub use error::*;

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

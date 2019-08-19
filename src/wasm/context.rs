// context.rs defines the context that is is passed around with default values
use web_sys::CanvasRenderingContext2d;

// The Context structure
#[derive(Debug)]
pub struct Context {
    pub values: Values,
}

impl Context {
    pub fn new() -> Self {
        Self {
            values: Values::new(),
        }
    }
}

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

// draw.rs contains helper functions for drawing to the canvas
use crate::game::{Die, Game, Hand, HAND_SIZE};
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

/// Draw a button
fn draw_button(
    text: &str,
    x: f64,
    y: f64,
    context: &CanvasRenderingContext2d,
    values: &Values,
) -> Result<(), JsValue> {
    context.begin_path();

    // Configure font
    context.set_font(&values.get_font_string());

    // Configure button size
    let text_width = context.measure_text(text).unwrap().width();
    let button_width = text_width + values.padding;
    let button_height = values.get_button_height();

    // Set color
    context.set_stroke_style(&JsValue::from_str(values.button_color));
    // Stage border
    context.rect(x, y, button_width, button_height);
    // Stage button text
    context.fill_text(text, x + (values.padding / 2.0), y + (button_height / 2.0))?;

    // Draw and return
    context.stroke();
    Ok(())
}

/// Draw some text
fn draw_text(
    text: &str,
    x: f64,
    y: f64,
    context: &CanvasRenderingContext2d,
) -> Result<(), JsValue> {
    context.begin_path();
    context.fill_text(text, x, y)?;
    context.stroke();
    Ok(())
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

// Trait representing things that can be drawn to the canvas
pub trait Drawable {
    // Draw this game element with the given top left corner
    fn draw_at(
        &self,
        x: f64,
        y: f64,
        context: &CanvasRenderingContext2d,
        values: &Values,
    ) -> Result<(), JsValue>;
}

impl Drawable for Die {
    fn draw_at(
        &self,
        x: f64,
        y: f64,
        context: &CanvasRenderingContext2d,
        values: &Values,
    ) -> Result<(), JsValue> {
        // draw a rectangle
        // if it's held, set the font color to red, otherwise black
        context.begin_path();
        context.rect(x, y, values.die_dimension, values.die_dimension);
        context.set_font("12px Arial");
        if self.held {
            context.set_stroke_style(&JsValue::from_str("red"));
        } else {
            context.set_stroke_style(&JsValue::from_str(values.button_color));
        }
        // TODO draw the dot pattern
        context.fill_text(
            &format!("{:?}", self.value),
            x + (values.padding / 2.0),
            y + (values.die_dimension / 2.0),
        )?;
        context.stroke();
        Ok(())
    }
}

impl Drawable for Hand {
    fn draw_at(
        &self,
        x: f64,
        y: f64,
        context: &CanvasRenderingContext2d,
        values: &Values,
    ) -> Result<(), JsValue> {
        // draw each die
        for (i, item) in self.dice.iter().enumerate().take(HAND_SIZE) {
            // draw each die taking into account offsets for die index and global game offset
            item.draw_at(
                values.dice_origin().0
                    + (i as f64 * (values.dice_origin().0 + values.die_dimension) + values.padding)
                    + x,
                values.dice_origin().1 + y,
                &context,
                values,
            )?;
        }

        // draw the Reroll button
        let reroll_button_corners = values.reroll_button_corners(context);
        let reroll_button_top_left = reroll_button_corners.0;
        let reroll_button_bottom_right = reroll_button_corners.1;
        draw_button(
            values.reroll_button_text,
            reroll_button_top_left.0,
            reroll_button_top_left.1,
            context,
            values,
        )?;

        // Draw remaining rolls readout
        let remaining_rolls_x = (reroll_button_bottom_right.0 - reroll_button_top_left.0)
            + (values.padding * 2.0)
            + values.padding;
        let remaining_rolls_y = reroll_button_bottom_right.1;
        draw_text(
            &format!("Rolls left: {}", self.remaining_rolls),
            remaining_rolls_x,
            remaining_rolls_y,
            context,
        )?;

        Ok(())
    }
}

impl Drawable for Game {
    fn draw_at(
        &self,
        x: f64,
        y: f64,
        context: &CanvasRenderingContext2d,
        values: &Values,
    ) -> Result<(), JsValue> {
        // draw current hand
        self.get_hand().draw_at(x, y, context, values)?;

        // draw start over button
        let start_over_top_left = values.start_over_button_corners(context).0;
        draw_button(
            "Start Over",
            start_over_top_left.0,
            start_over_top_left.1,
            context,
            values,
        )?;

        Ok(())
    }
}

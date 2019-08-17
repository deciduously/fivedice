// draw.rs contains helper functions for drawing to the canvas
use crate::game::{Die, Game, HAND_SIZE};
use wasm_bindgen::JsValue;

/// Layout values
#[derive(Debug, Clone, Copy)]
pub struct Values {
    pub canvas_size: (u32, u32),
    pub die_dimension: f64,
    pub dice_origin: (f64, f64),
}

impl Values {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Values {
    fn default() -> Self {
        Self {
            canvas_size: (640, 480),
            die_dimension: 40.0,
            dice_origin: (10.0, 20.0),
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
        context: &web_sys::CanvasRenderingContext2d,
        values: &Values,
    ) -> Result<(), JsValue>;
}

impl Drawable for Die {
    fn draw_at(
        &self,
        x: f64,
        y: f64,
        context: &web_sys::CanvasRenderingContext2d,
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
            context.set_stroke_style(&JsValue::from_str("black"));
        }
        // TODO draw the dot pattern
        context.fill_text(
            &format!("{:?}", self.value),
            x + 5.0,
            y + (values.die_dimension / 2.0),
        )?;
        context.stroke();
        Ok(())
    }
}

impl Drawable for Game {
    fn draw_at(
        &self,
        x: f64,
        y: f64,
        context: &web_sys::CanvasRenderingContext2d,
        values: &Values,
    ) -> Result<(), JsValue> {
        // draw each die
        // TODO factor this out so that handle_click can reference the same data as draw_at
        let dice = self.get_hand();
        for (i, item) in dice.iter().enumerate().take(HAND_SIZE) {
            item.draw_at(
                values.dice_origin.0
                    + (i as f64 * (values.dice_origin.0 + values.die_dimension))
                    + x,
                values.dice_origin.1 + y,
                &context,
                values,
            )?;
        }

        Ok(())
    }
}

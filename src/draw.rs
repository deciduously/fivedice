// draw.rs contains helper functions for drawing to the canvas
use crate::game::{Die, Game, HAND_SIZE};
use wasm_bindgen::JsValue;

// Trait representing things that can be drawn to the canvas
pub trait Drawable {
    // Draw this game element with the given top left corner
    fn draw_at(
        &self,
        x: f64,
        y: f64,
        context: &web_sys::CanvasRenderingContext2d,
    ) -> Result<(), JsValue>;
}

impl Drawable for Die {
    fn draw_at(
        &self,
        x: f64,
        y: f64,
        context: &web_sys::CanvasRenderingContext2d,
    ) -> Result<(), JsValue> {
        // draw a rectangle
        // if it's held, set the font color to red, otherwise black

        context.begin_path();
        context.rect(x, y, 40.0, 40.0);
        context.set_font("12px Arial");
        if self.held {
            context.set_stroke_style(&JsValue::from_str("red"));
        } else {
            context.set_stroke_style(&JsValue::from_str("black"));
        }
        // TODO draw the dot pattern
        context.fill_text(&format!("{:?}", self.value), x + 5.0, y + 20.0)?;
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
    ) -> Result<(), JsValue> {
        // draw each die
        let dice = self.get_hand();
        for (i, item) in dice.iter().enumerate().take(HAND_SIZE) {
            item.draw_at((10 + (i * 50)) as f64 + x, 20.0 + y, &context)?;
        }

        Ok(())
    }
}

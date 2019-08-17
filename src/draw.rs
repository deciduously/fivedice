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

        let dim = 40.0;

        context.begin_path();
        context.rect(x, y, dim, dim);
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
        // TODO factor this out so that handle_click can reference the same data as draw_at
        let dice_start_x = 10.0;
        let dice_start_y = 20.0;
        let dice_dim = 40.0;
        let dice = self.get_hand();
        for (i, item) in dice.iter().enumerate().take(HAND_SIZE) {
            item.draw_at(
                dice_start_x + (i as f64 * (dice_start_x + dice_dim)) + x,
                dice_start_y + y,
                &context,
            )?;
        }

        Ok(())
    }
}

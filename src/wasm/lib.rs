// lib.rs - Five Dice WASM module top-level

use wasm_bindgen::prelude::*;

// Error type
mod error;
// Game logic
mod game;

use crate::game::{FiveDiceMessage, Game};
use widget_grid::window::{WebSysCanvas, WindowEngine};

/// Entry point for the module
#[allow(dead_code)]
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    // Instantiate canvas
    let renderable_context =
        Box::new(WebSysCanvas::new("Five Dice").expect("Should instantiate canvas window engine"));

    // Instantiate game
    let game = Box::new(Game::new());

    // Instantiate engine
    let engine: WindowEngine<FiveDiceMessage> = WindowEngine::new(renderable_context, game);

    // Run game
    engine.start();
}

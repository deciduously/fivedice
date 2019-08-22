// lib.rs - Five Dice WASM module top-level

#[macro_use]
#[doc(hidden)]
extern crate lazy_static;

use wasm_bindgen::prelude::*;

// Canvas drawing
mod draw;
// Error type
mod error;
// Game logic
mod game;

use crate::{
    draw::{WebSysCanvas, WindowEngine},
    game::Game,
};

/// Entry point for the module
#[allow(dead_code)]
#[wasm_bindgen(start)]
pub fn start() {
    // Instantiate canvas
    let renderable_context =
        Box::new(WebSysCanvas::new("Five Dice").expect("Could not instantiate canvas"));

    // Instantiate game
    let game = Box::new(Game::new());

    // Instantiate engine
    let engine = WindowEngine::new(renderable_context, game);

    // Run game
    engine.start();
}

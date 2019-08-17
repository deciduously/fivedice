// lib.rs - Five Dice WASM module top-level

// DOM manipultion macros
#[macro_use]
mod dom;
// JS<->Rust Interop
mod ffi;
// Game logic
mod game;

// Game screen dimensions
const CANVAS_X: u32 = 800;
const CANVAS_Y: u32 = 600;

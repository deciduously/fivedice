// lib.rs - Five Dice WASM module top-level

// DOM manipultion macros
#[macro_use]
mod dom;
// Canvas drawing
mod draw;
// JS<->Rust Interop
mod ffi;
// Game logic
mod game;

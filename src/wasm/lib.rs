// lib.rs - Five Dice WASM module top-level

#[macro_use]
extern crate lazy_static;

// DOM manipulation macros
#[macro_use]
mod dom;
// Canvas drawing
mod draw;
// Error type
mod error;
// JS<->Rust Interop
mod ffi;
// Game logic
mod game;

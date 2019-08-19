// lib.rs - Five Dice WASM module top-level

// Context object for storing canvas reference and layout config
mod context;
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

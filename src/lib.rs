#[macro_use]
mod dom;
mod ffi;
mod game;

use ffi::{get_document, Result};
use game::Game;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn start() -> Result<()> {
    let game = Game::new();
    let document = get_document()?;
    let body = document.body().unwrap();
    append_text_element_attrs!(document, body, "h1", "FIVE DICE",);
    append_text_element_attrs!(document, body, "span", &format!("{}", game),);
    Ok(())
}

// ffi.rs contains all JS<->Rust interfaces

use crate::game::Player;
use js_sys::Math::{floor, random};
use std::{fmt, str::FromStr};
use wasm_bindgen::prelude::*;
use web_sys::Document;

/// Grab the document
pub fn get_document() -> Result<Document, JsValue> {
    let window = web_sys::window().unwrap();
    Ok(window.document().unwrap())
}

/// use js Math.random() to get an integer in range [min, max)
pub fn js_gen_range(min: i64, max: i64) -> i64 {
    (floor(random() * (max as f64 - min as f64)) + min as f64) as i64
}

// All the various ways the game can be interacted with
pub enum Message {
    HoldDie(usize),
}

impl FromStr for Message {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Message::HoldDie(1)) // OBVIOUSLY TODO
    }
}

/// Controls for mounting
#[derive(Debug)]
enum GameState {
    Playing,
    Unmounted,
}

/// The Game object
#[wasm_bindgen]
#[repr(C)]
#[derive(Debug)]
pub struct Game {
    gamestate: GameState,
    player: Player,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            gamestate: GameState::Unmounted,
            player: Player::new(),
        }
    }

    // Toggle one die on the player
    fn hold_die(&mut self, die_idx: usize) {
        if die_idx < self.player.current_hand.dice.len() {
            self.player.current_hand.dice[die_idx].toggle_held();
        }
    }

    /// Redraw the screen
    pub fn draw(&self) -> Result<(), JsValue> {
        let document = get_document()?;
        let display_output = document.query_selector("#display-output")?.unwrap();
        display_output.set_text_content(Some(&format!("{}", self)));
        Ok(())
    }

    /// Mount the DOM representation
    // TODO only do once- set a state?
    pub fn mount(&mut self) -> Result<(), JsValue> {
        match self.gamestate {
            GameState::Unmounted => {
                let document = get_document()?;
                let body = document.body().unwrap();
                append_text_element_attrs!(document, body, "h1", "FIVE DICE",);
                append_text_element_attrs!(
                    document,
                    body,
                    "span",
                    &format!("{}", self),
                    ("id", "display-output")
                );
                self.gamestate = GameState::Playing;
                Ok(())
            }
            // TODO Error??
            _ => Ok(()),
        }
    }

    /// Handle all incoming messages
    /// TODO send an outgoing result?
    pub fn reducer(&mut self, msg_str: &str) {
        use Message::*;
        if let Ok(res) = Message::from_str(msg_str) {
            match res {
                HoldDie(idx) => self.hold_die(idx),
            }
        }
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.player)
    }
}

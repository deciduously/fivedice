// game.rs contains the game logic

use crate::draw::Drawable;
use crate::ffi::{get_document, js_gen_range};

use std::fmt;
use wasm_bindgen::{prelude::*, JsCast};

// Number of dice in a turn
pub const HAND_SIZE: usize = 5;

/// A single player's score object
#[derive(Debug)]
struct Score {
    ones: Option<u8>,
    twos: Option<u8>,
    threes: Option<u8>,
    fours: Option<u8>,
    fives: Option<u8>,
    sixes: Option<u8>,
    three_kind: bool,
    four_kind: bool,
    full_house: bool,
    sm_straight: bool,
    lg_straight: bool,
    yahtzee: bool,
    bonus_yahtzee: Option<u8>,
    chance: Option<u8>,
}

impl Score {
    fn new() -> Self {
        Self::default()
    }
}

impl Default for Score {
    fn default() -> Self {
        Self {
            ones: None,
            twos: None,
            threes: None,
            fours: None,
            fives: None,
            sixes: None,
            three_kind: false,
            four_kind: false,
            full_house: false,
            sm_straight: false,
            lg_straight: false,
            yahtzee: false,
            bonus_yahtzee: None,
            chance: None,
        }
    }
}

impl fmt::Display for Score {
    // TODO make this nice - low priority
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SCORE")
    }
}

/// Each possible Die result
#[derive(Debug, Clone, Copy)]
pub enum RollResult {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
}

/// A single Die, can be held or not
#[derive(Debug, Clone, Copy)]
pub struct Die {
    pub value: RollResult,
    pub held: bool,
}

impl Die {
    fn new(value: RollResult) -> Self {
        Self { value, held: false }
    }

    /// Get a random die
    fn get_random() -> Self {
        use RollResult::*;
        match js_gen_range(1, 7) {
            1 => Self::new(One),
            2 => Self::new(Two),
            3 => Self::new(Three),
            4 => Self::new(Four),
            5 => Self::new(Five),
            6 => Self::new(Six),
            _ => unreachable!(),
        }
    }

    /// Toggles whether this die is held
    pub fn toggle_held(&mut self) {
        self.held = !self.held;
    }
}

impl fmt::Display for Die {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}{}", self.value, (if self.held { "X" } else { "" }))
    }
}

/// A set of 5 dice for a single play
#[derive(Debug)]
struct Hand {
    dice: [Die; HAND_SIZE],
}

impl Hand {
    fn new() -> Self {
        Self {
            // HAND_SIZE is hard-coded to 5 - this doesn't work otherwise
            dice: [
                Die::get_random(),
                Die::get_random(),
                Die::get_random(),
                Die::get_random(),
                Die::get_random(),
            ],
        }
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} | {} | {} | {} | {}",
            self.dice[0], self.dice[1], self.dice[2], self.dice[3], self.dice[4]
        )
    }
}

/// The Player object
#[derive(Debug)]
struct Player {
    score: Score,
    current_hand: Hand,
}

impl Player {
    fn new() -> Self {
        Self {
            current_hand: Hand::new(),
            score: Score::new(),
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} || {}", self.current_hand, self.score)
    }
}

// All the various ways the game can be interacted with
enum Message {
    HoldDie(usize),
}

/// The Game object
#[derive(Debug)]
pub struct Game {
    player: Player,
}

impl Game {
    pub fn new() -> Self {
        Self::default()
    }

    /// Handle a click at canvasX, canvasY
    pub fn handle_click(&mut self, canvas_x: f64, canvas_y: f64) {
        use Message::*;
        // Check if it hit a die
        let dice_dim = 40.0;
        let dice_start_x = 10.0;
        let dice_start_y = 20.0;
        let dice_padding = dice_dim + dice_start_x;
        for i in 0..HAND_SIZE {
            let die_start_x = dice_start_x + (dice_padding * i as f64);
            let die_end_x = dice_start_x + dice_dim + (dice_padding * i as f64);
            if canvas_x >= die_start_x
                && canvas_x <= die_end_x
                && canvas_y >= dice_start_y
                && canvas_y <= dice_start_y + dice_dim
            {
                self.reducer(HoldDie(i));
            }
        }
    }

    // Return all the current dice in play
    pub fn get_hand(&self) -> [Die; HAND_SIZE] {
        self.player.current_hand.dice
    }

    // Toggle one die on the player
    fn hold_die(&mut self, die_idx: usize) {
        if die_idx < HAND_SIZE {
            self.player.current_hand.dice[die_idx].toggle_held();
        }
    }

    /// Redraw the screen
    pub fn draw(&self) -> Result<(), JsValue> {
        let document = get_document();
        let canvas = document
            .query_selector("canvas")?
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()?;
        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
        context.clear_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());
        self.draw_at(0.0, 0.0, &context)?;
        Ok(())
    }

    /// Handle all incoming messages
    /// TODO send an outgoing result?  Maybe use the memory tape for streaming events back
    fn reducer(&mut self, msg: Message) {
        use Message::*;
        match msg {
            HoldDie(idx) => self.hold_die(idx),
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self {
            player: Player::new(),
        }
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.player)
    }
}

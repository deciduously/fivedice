// game.rs contains the game logic

use crate::draw::{Drawable, Values};
use crate::ffi::{get_context, js_gen_range};

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
    two_and_three: bool,
    sm_straight: bool,
    lg_straight: bool,
    five_dice: bool,
    five_dice_again: Option<u8>,
    stone_soup: Option<u8>,
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
            two_and_three: false,
            sm_straight: false,
            lg_straight: false,
            five_dice: false,
            five_dice_again: None,
            stone_soup: None,
        }
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
        Self::new(Self::get_random_result())
    }

    /// Get a random result
    fn get_random_result() -> RollResult {
        use RollResult::*;
        match js_gen_range(1, 7) {
            1 => One,
            2 => Two,
            3 => Three,
            4 => Four,
            5 => Five,
            6 => Six,
            _ => unreachable!(),
        }
    }

    /// Roll this die - no acction if currently held
    pub fn roll(&mut self) {
        if !self.held {
            self.value = Self::get_random_result();
        }
    }

    /// Toggles whether this die is held
    pub fn toggle_held(&mut self) {
        self.held = !self.held;
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

// All the various ways the game can be interacted with
enum Message {
    HoldDie(usize),
    RollDice,
}

/// The Game object
#[derive(Debug)]
pub struct Game {
    // For now, just a solo game
    player: Player,
    // The layout constants to use
    pub values: Values,
}

impl Game {
    pub fn new() -> Self {
        Self::default()
    }

    // If the given coordinates fall in a given region execute f
    fn detect_region(
        &mut self,
        x: f64,
        y: f64,
        top_left_x: f64,
        top_left_y: f64,
        bottom_right_x: f64,
        bottom_right_y: f64,
    ) -> bool {
        if x >= top_left_x && x <= bottom_right_x && y >= top_left_y && y <= bottom_right_y {
            true
        } else {
            false
        }
    }

    /// Handle a click at canvasX, canvasY
    pub fn handle_click(&mut self, click_x: f64, click_y: f64) {
        use Message::*;
        // Check if it hit a die
        // grab relevant dimensions from the values struct
        let dice_dim = self.values.die_dimension;
        let dice_start_x = self.values.dice_origin.0;
        let dice_start_y = self.values.dice_origin.1;
        let dice_padding = dice_dim + self.values.padding;
        // check if hit given is in each die's boundary
        for i in 0..HAND_SIZE {
            let die_start_x = dice_start_x + (dice_padding * i as f64);
            let die_end_x = dice_start_x + dice_dim + (dice_padding * i as f64);
            let die_end_y = dice_start_y + dice_dim;
            if self.detect_region(
                click_x,
                click_y,
                die_start_x,
                dice_start_y,
                die_end_x,
                die_end_y,
            ) {
                self.reducer(HoldDie(i));
            }
        }

        // check if we hit the Roll button
        let roll_button_corners = self.values.reroll_button_corners(&get_context());
        let top_left = roll_button_corners.0;
        let bottom_right = roll_button_corners.1;
        if self.detect_region(
            click_x,
            click_y,
            top_left.0,
            top_left.1,
            bottom_right.0,
            bottom_right.1,
        ) {
            self.reducer(RollDice);
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
        let context = get_context();
        context.clear_rect(
            0.0,
            0.0,
            self.values.canvas_size.0.into(),
            self.values.canvas_size.1.into(),
        );
        self.draw_at(0.0, 0.0, &context, &self.values)?;
        Ok(())
    }

    /// Handle all incoming messages
    /// TODO send an outgoing result?  Maybe use the memory tape for streaming events back
    fn reducer(&mut self, msg: Message) {
        use Message::*;
        match msg {
            HoldDie(idx) => self.hold_die(idx),
            RollDice => self.roll_dice(),
        }
    }

    /// Roll all unheld dice
    fn roll_dice(&mut self) {
        for die in self.player.current_hand.dice.iter_mut() {
            die.roll();
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self {
            player: Player::new(),
            values: Values::new(),
        }
    }
}

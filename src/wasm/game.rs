// game.rs contains the game logic

use js_sys::Math::{floor, random};
use std::rc::Rc;
use widget_grid::{
    window::WindowPtr, Button, Drawable, Message, MountedWidget, Point, Region, Text, Widget,
    VALUES,
};

type WindowResult<T> = widget_grid::Result<T>;

/// use js Math.random() to get an integer in range [min, max)
pub fn js_gen_range(min: i64, max: i64) -> i64 {
    (floor(random() * (max as f64 - min as f64)) + min as f64) as i64
}

// Number of dice in a turn
pub const HAND_SIZE: usize = 5;

/// A single player's score object
#[derive(Debug, Default)]
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
    value: RollResult,
    held: bool,
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

    /// Roll this die - no action if currently held
    fn roll(&mut self) {
        if !self.held {
            self.value = Self::get_random_result();
        }
    }

    /// Toggles whether this die is held
    fn toggle_held(&mut self) {
        self.held = !self.held;
    }
}

// TODO make it easy to impl Widget for items that are Drawable already
// I smell a macro DSL?  Just one variadic macro should do it at first

impl Widget for Die {
    fn mount_widget(&self) -> MountedWidget {
        let mut ret = MountedWidget::new();
        ret.set_drawable(Box::new(Button::new(
            &format!("{:?}", self.value),
            Some((VALUES.die_dimension, VALUES.die_dimension).into()),
            || None,
        )));
        ret
    }
    fn get_region(&self, top_left: Point, w: WindowPtr) -> WindowResult<Region> {
        self.mount_widget().get_region(top_left, w)
    }
}

/// A set of 5 dice for a single play
#[derive(Debug, Clone, Copy)]
pub struct Hand {
    pub dice: [Die; HAND_SIZE],
    pub remaining_rolls: u8,
}

impl Hand {
    fn new() -> Self {
        Self::default()
    }

    /// all unheld dice if there are rolls left
    pub fn roll(&mut self) {
        if self.remaining_rolls > 0 {
            for die in self.dice.iter_mut() {
                die.roll();
            }
            self.remaining_rolls -= 1;
        }
    }
}

impl Default for Hand {
    fn default() -> Self {
        Self {
            // HAND_SIZE is hard-coded to 5 - this doesn't work otherwise
            dice: [
                Die::get_random(),
                Die::get_random(),
                Die::get_random(),
                Die::get_random(),
                Die::get_random(),
            ],
            remaining_rolls: 3,
        }
    }
}

impl Widget for Hand {
    fn mount_widget(&self) -> MountedWidget {
        let mut ret = MountedWidget::new();
        for die in &self.dice {
            ret.push_current_row(Box::new(*die));
        }
        ret.push_new_row(Box::new(Button::new(
            VALUES.reroll_button_text,
            None,
            || Some(Box::new(FiveDiceMessage::RollDice)),
        )));
        ret.push_current_row(Box::new(Text::new(&format!(
            "Remaining rolls: {}",
            self.remaining_rolls
        ))));
        ret
    }
    fn get_region(&self, top_left: Point, w: WindowPtr) -> WindowResult<Region> {
        let mut ret = (top_left, 0.0, 0.0).into();
        for die in &self.dice {
            ret += die.mount_widget().get_region(top_left, Rc::clone(&w))?;
        }
        Ok(ret)
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

    fn get_hand(&self) -> Box<Hand> {
        Box::new(self.current_hand)
    }
}

// All the various ways the game can be interacted with
#[derive(Debug, Clone, Copy)]
enum FiveDiceMessage {
    HoldDie(usize),
    RollDice,
    StartOver,
}

impl Message for FiveDiceMessage {}

/// The Game object
#[derive(Debug)]
pub struct Game {
    // For now, just a solo game
    player: Player,
}

impl Game {
    pub fn new() -> Self {
        Self {
            player: Player::new(),
        }
    }

    // Detect if the given coordinates fall in a given region
    fn detect_region(
        &mut self,
        x: f64,
        y: f64,
        top_left_x: f64,
        top_left_y: f64,
        bottom_right_x: f64,
        bottom_right_y: f64,
    ) -> bool {
        x >= top_left_x && x <= bottom_right_x && y >= top_left_y && y <= bottom_right_y
    }

    // Return all the current dice in play
    pub fn get_hand(&self) -> Hand {
        self.player.current_hand
    }

    // Toggle one die on the player
    fn hold_die(&mut self, die_idx: usize) {
        if die_idx < HAND_SIZE {
            self.player.current_hand.dice[die_idx].toggle_held();
        }
    }

    /// Handle all incoming messages
    /// TODO send an outgoing result?  Maybe use the memory tape for streaming events back
    fn reducer(&mut self, msg: FiveDiceMessage) {
        use FiveDiceMessage::*;
        match msg {
            HoldDie(idx) => self.hold_die(idx),
            RollDice => self.roll_dice(),
            StartOver => self.reset(),
        }
    }

    /// Start a fresh new game
    fn reset(&mut self) {
        self.player = Player::new();
    }

    /// Roll all unheld dice
    fn roll_dice(&mut self) {
        self.player.current_hand.roll();
    }
}

impl Widget for Game {
    fn mount_widget(&self) -> MountedWidget {
        let mut ret = MountedWidget::new();
        ret.push_current_row(self.player.get_hand());
        ret
    }
    fn get_region(&self, top_left: Point, w: WindowPtr) -> WindowResult<Region> {
        self.player.get_hand().get_region(top_left, w)
    }
}

// game.rs contains the game logic

use js_sys::Math::{floor, random};
use std::{rc::Rc, str::FromStr};
//use web_sys::console;
use widget_grid::{
    traits::{MountedWidget, Widget},
    types::{Callback, Color, Point, Region},
    widgets::{Button, Text},
    window::WindowPtr,
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
    id: u8,
    value: RollResult,
    held: bool,
}

impl Die {
    fn new(id: u8, value: RollResult) -> Self {
        Self {
            id,
            value,
            held: false,
        }
    }

    /// Get a random die
    fn get_random(id: u8) -> Self {
        Self::new(id, Self::get_random_result())
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
    type MSG = FiveDiceMessage;
    fn mount_widget(&self) -> MountedWidget<Self::MSG> {
        let mut ret = MountedWidget::new();
        // Will get moved into closure - cannot call self inside, lifetime conflict (need 'static)
        let id = self.id as usize;
        let die_color = if self.held {
            Color::from_str("red").unwrap()
        } else {
            Color::from_str("black").unwrap()
        };
        let button = Button::new(
            &format!("{:?}", self.value),
            Some((VALUES.die_dimension, VALUES.die_dimension).into()),
            die_color,
            Some(Callback::from(move || -> FiveDiceMessage {
                FiveDiceMessage::HoldDie(id)
            })),
        );
        ret.push_current_row(Box::new(button));
        ret
    }
    fn get_region(&self, top_left: Point, w: WindowPtr) -> WindowResult<Region> {
        let mw: MountedWidget<Self::MSG> = self.mount_widget();
        mw.get_region(top_left, w)
    }
    fn handle_click(
        &mut self,
        top_left: Point,
        click: Point,
        w: WindowPtr,
    ) -> WindowResult<Option<Self::MSG>> {
        // TODO this is identical to hand, no need to write every time
        let mut mw: MountedWidget<Self::MSG> = self.mount_widget();
        let msg = mw.click(top_left, click, w)?;
        if msg.is_some() {
            Ok(msg)
        } else {
            Ok(None)
        }
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
            // HAND_SIZE is hard-coded to 5 - TODO do this in a loop?
            dice: [
                Die::get_random(0),
                Die::get_random(1),
                Die::get_random(2),
                Die::get_random(3),
                Die::get_random(4),
            ],
            remaining_rolls: 3,
        }
    }
}

impl Widget for Hand {
    type MSG = FiveDiceMessage;
    fn mount_widget(&self) -> MountedWidget<Self::MSG> {
        let mut ret = MountedWidget::new();
        for die in &self.dice {
            ret.push_current_row(Box::new(*die));
        }
        // TODO the reroll button only picks up clicks on the bottom half of the button
        ret.push_new_row(Box::new(Button::new(
            VALUES.reroll_button_text,
            None,
            Color::from_str("black").unwrap(),
            Some(Callback::from(|| -> Self::MSG {
                FiveDiceMessage::RollDice
            })),
        )));
        ret.push_current_row(Box::new(Text::new(&format!(
            "Remaining rolls: {}",
            self.remaining_rolls
        ))));
        ret
    }
    fn get_region(&self, top_left: Point, w: WindowPtr) -> WindowResult<Region> {
        let mut ret = (top_left, 0.0, 0.0).into();
        let mut cursor = top_left;
        for die in &self.dice {
            // You've got to use a cursor
            let mw: MountedWidget<Self::MSG> = die.mount_widget();
            let region = mw.get_region(cursor, Rc::clone(&w))?;
            ret += region;
            cursor = region.top_right();
            cursor.horiz_offset(VALUES.padding)?;
        }
        Ok(ret)
    }
    fn handle_click(
        &mut self,
        top_left: Point,
        click: Point,
        w: WindowPtr,
    ) -> WindowResult<Option<Self::MSG>> {
        let mut mw: MountedWidget<Self::MSG> = self.mount_widget();
        let msg = mw.click(top_left, click, Rc::clone(&w))?;
        if msg.is_some() {
            Ok(msg)
        } else {
            Ok(None)
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

    fn get_hand(&self) -> Box<Hand> {
        Box::new(self.current_hand)
    }
}

// All the various ways the game can be interacted with
#[derive(Debug, Clone, Copy)]
pub enum FiveDiceMessage {
    HoldDie(usize),
    RollDice,
    StartOver,
}

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
    type MSG = FiveDiceMessage;
    fn mount_widget(&self) -> MountedWidget<Self::MSG> {
        let mut ret = MountedWidget::new();
        ret.push_current_row(Box::new(Button::new(
            "Start Over",
            None,
            Color::from_str("black").unwrap(),
            Some(Callback::from(|| -> Self::MSG {
                FiveDiceMessage::StartOver
            })),
        )));
        ret.push_new_row(self.player.get_hand());
        ret
    }
    fn get_region(&self, top_left: Point, w: WindowPtr) -> WindowResult<Region> {
        let mw: MountedWidget<Self::MSG> = self.player.get_hand().mount_widget();
        mw.get_region(top_left, w)
    }
    fn handle_click(
        &mut self,
        top_left: Point,
        click: Point,
        w: WindowPtr,
    ) -> WindowResult<Option<Self::MSG>> {
        // Mount the widget and collect any message for this click point
        let mut mw: MountedWidget<Self::MSG> = self.mount_widget();
        let msg = mw.click(top_left, click, w)?;
        if let Some(m) = msg {
            // Handle the click
            self.reducer(m);
        }
        // Nothing to pass up to the caller
        Ok(None)
    }
}

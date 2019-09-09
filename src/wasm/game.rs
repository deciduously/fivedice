// game.rs contains the game logic

use js_sys::Math::{floor, random};
use std::{collections::HashSet, rc::Rc, str::FromStr};
//use web_sys::console;
use widget_grid::{
    traits::{MountedWidget, Widget},
    types::{Callback, Color, Point},
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

/// Each possible option
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ScoreType {
    Ones(u8),
    Twos(u8),
    Threes(u8),
    Fours(u8),
    Fives(u8),
    Sixes(u8),
    ThreeKind,
    FourKind,
    TwoAndThree,
    SmStraight,
    LgStraight,
    AllFive,
    AllFiveBonus(u8),
    StoneSoup(u8),
}

impl ScoreType {
    /// Return whether this score can be taken from the current hand
    fn isValid(&self, hand: &Hand) -> bool {
        unimplemented!()
        // match self...
    }
}

/// A single score option
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ScoreSlot {
    taken: bool,
    value: ScoreType,
}

impl ScoreSlot {
    fn new(value: ScoreType) -> Self {
        Self {
            taken: false,
            value,
        }
    }
}

impl Widget for ScoreSlot {
    type MSG = FiveDiceMessage;
    fn mount_widget(&self) -> MountedWidget<Self::MSG> {
        let mut ret = MountedWidget::new();
        ret.push_current_row(Box::new(Text::new(&format!("{:?}", self))));
        ret
    }
    fn handle_click(
        &mut self,
        top_left: Point,
        click: Point,
        w: WindowPtr,
    ) -> WindowResult<Option<Self::MSG>> {
        Ok(None)
    }
}

/// A single player's score object
#[derive(Debug, Clone)]
struct Score {
    slots: HashSet<ScoreSlot>,
}

impl Score {
    fn new() -> Self {
        Self::default()
    }
}

impl Default for Score {
    fn default() -> Self {
        use ScoreType::*;
        let mut ret = Self {
            slots: HashSet::new(),
        };
        ret.slots.insert(ScoreSlot::new(Ones(0)));
        ret.slots.insert(ScoreSlot::new(Twos(0)));
        ret.slots.insert(ScoreSlot::new(Threes(0)));
        ret.slots.insert(ScoreSlot::new(Fours(0)));
        ret.slots.insert(ScoreSlot::new(Fives(0)));
        ret.slots.insert(ScoreSlot::new(Sixes(0)));
        ret.slots.insert(ScoreSlot::new(ThreeKind));
        ret.slots.insert(ScoreSlot::new(FourKind));
        ret.slots.insert(ScoreSlot::new(TwoAndThree));
        ret.slots.insert(ScoreSlot::new(SmStraight));
        ret.slots.insert(ScoreSlot::new(LgStraight));
        ret.slots.insert(ScoreSlot::new(AllFive));
        ret.slots.insert(ScoreSlot::new(AllFiveBonus(0)));
        ret.slots.insert(ScoreSlot::new(StoneSoup(0)));
        ret
    }
}

impl Widget for Score {
    type MSG = FiveDiceMessage;
    fn mount_widget(&self) -> MountedWidget<Self::MSG> {
        let mut ret = MountedWidget::new();
        // first in first row
        for slot in &self.slots {
            ret.push_new_row(Box::new(*slot));
        }
        ret
    }
    fn handle_click(
        &mut self,
        top_left: Point,
        click: Point,
        w: WindowPtr,
    ) -> WindowResult<Option<Self::MSG>> {
        Ok(None)
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
        let mut button = Button::new(&format!("{:?}", self.value));
        button.add_border_color(die_color);
        button.set_onclick(Callback::from(move || -> FiveDiceMessage {
            FiveDiceMessage::HoldDie(id)
        }));
        button.set_size(VALUES.die_dimension, VALUES.die_dimension);
        ret.push_current_row(Box::new(button));
        ret
    }
    fn handle_click(
        &mut self,
        top_left: Point,
        click: Point,
        w: WindowPtr,
    ) -> WindowResult<Option<Self::MSG>> {
        // TODO this is identical to hand, no need to write every time
        let mut mw: MountedWidget<Self::MSG> = self.mount_widget();
        Ok(mw.click(top_left, click, w)?)
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
        let mut button = Button::new(VALUES.reroll_button_text);
        button.set_onclick(Callback::from(|| -> Self::MSG {
            FiveDiceMessage::RollDice
        }));
        ret.push_new_row(Box::new(button));
        ret.push_current_row(Box::new(Text::new(&format!(
            "Remaining rolls: {}",
            self.remaining_rolls
        ))));
        ret
    }
    fn handle_click(
        &mut self,
        top_left: Point,
        click: Point,
        w: WindowPtr,
    ) -> WindowResult<Option<Self::MSG>> {
        let mut mw: MountedWidget<Self::MSG> = self.mount_widget();
        mw.click(top_left, click, Rc::clone(&w))
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
    score: Score,
}

impl Game {
    pub fn new() -> Self {
        Self {
            player: Player::new(),
            score: Score::new(),
        }
    }

    /// Get a pointer to the current score
    fn get_score(&self) -> &Score {
        &self.score
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
        let mut button = Button::new("Start Over");
        button.set_onclick(Callback::from(|| -> Self::MSG {
            FiveDiceMessage::StartOver
        }));
        ret.push_current_row(Box::new(button));
        ret.push_new_row(self.player.get_hand());
        // TODO Hand is overlapping - looks like it doesn't notice the actual bottom_right for the hand widget, just the text
        ret.push_new_row(Box::new(self.get_score().clone()));
        ret
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

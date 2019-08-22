// game.rs contains the game logic

use js_sys::Math::{floor, random};
use std::str::FromStr;
use widget_grid::{
    window::{WindowPtr, WindowResult},
    Color, Drawable, MountedWidget, Point, Region, Text, Widget, VALUES,
};

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

// TODO this will likely disappear and ust be bButtons added to the Widget impl
impl Drawable for Die {
    fn draw_at(&self, top_left: Point, w: WindowPtr) -> WindowResult<Point> {
        // draw a rectangle
        // if it's held, set the font color to red, otherwise black
        w.begin_path();
        let outline_region = Drawable::get_region(self, top_left);
        w.rect(outline_region);
        if self.held {
            w.set_color(Color::from_str("red")?);
        } else {
            w.set_color(Color::from_str(VALUES.button_color)?);
        }
        // TODO draw the dot pattern
        w.text(
            &format!("{:?}", self.value),
            &VALUES.get_font_string(),
            (
                top_left.x + (VALUES.padding / 2.0),
                top_left.y + (VALUES.die_dimension / 2.0),
            )
                .into(),
        )?;
        w.draw_path();
        Ok(outline_region.bottom_right())
    }

    fn get_region(&self, top_left: Point) -> Region {
        (top_left, VALUES.die_dimension, VALUES.die_dimension).into()
    }
}

// TODO make it easy to impl Widget for items that are Drawable already
// I smell a macro DSL?  Just one variadic macro should do it at first

impl Widget for Die {
    fn mount_widget(&self, top_left: Point) -> MountedWidget {
        let mut ret = MountedWidget::new(top_left);
        // TODO remove the text from the Drawable, use a Text node pushed to children
        ret.set_drawable(Box::new(*self));
        ret
    }
    fn get_region(&self, top_left: Point) -> Region {
        Drawable::get_region(self, top_left)
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
    fn mount_widget(&self, top_left: Point) -> MountedWidget {
        let mut ret = MountedWidget::new(top_left);
        for die in &self.dice {
            ret.push_current_row(Box::new(*die));
        }
        // TODO add Reroll Button
        ret.push_new_row(Box::new(Text::new(&format!(
            "Remaining rolls: {}",
            self.remaining_rolls
        ))));
        ret
    }
    fn get_region(&self, top_left: Point) -> Region {
        let mut ret = Region::default();
        for die in &self.dice {
            ret += Drawable::get_region(die, top_left);
        }
        ret
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
enum Message {
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
    /*
        /// Handle a click at canvasX, canvasY
        pub fn handle_click(
            &mut self,
            click_x: f64,
            click_y: f64,
            ctx: &web_sys::CanvasRenderingContext2d,
        ) {
            use Message::*;
            // Will be moved to Clickable, bot for now...
            let values = self.context.values;
            // Check if it hit a die
            // grab relevant dimensions from the values struct
            let dice_dim = values.die_dimension;
            let dice_start_x = values.dice_origin().0;
            let dice_start_y = values.dice_origin().1;
            let dice_padding = dice_dim + values.padding;
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
            let (top_left, bottom_right) = values.reroll_button_corners(&ctx);
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

            // check if we hit the Start Over button
            let (top_left, bottom_right) = values.start_over_button_corners(&ctx);
            if self.detect_region(
                click_x,
                click_y,
                top_left.0,
                top_left.1,
                bottom_right.0,
                bottom_right.1,
            ) {
                self.reducer(StartOver);
            }
        }
    */
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
    fn reducer(&mut self, msg: Message) {
        use Message::*;
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
    fn mount_widget(&self, top_left: Point) -> MountedWidget {
        let mut ret = MountedWidget::new(top_left);
        ret.push_current_row(self.player.get_hand());
        ret
    }
    fn get_region(&self, top_left: Point) -> Region {
        self.player.get_hand().get_region(top_left)
    }
}

// game.rs contains the game logic

use crate::ffi::js_gen_range;

use std::fmt;

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
    // TODO make this nice - low prio
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SCORE")
    }
}

/// Each possible Die result
#[derive(Debug)]
enum RollResult {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
}

/// A single Die, can be held or not
#[derive(Debug)]
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
pub struct Hand {
    pub dice: [Die; 5],
}

impl Hand {
    fn new() -> Self {
        Self {
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
pub struct Player {
    score: Score,
    pub current_hand: Hand,
}

impl Player {
    pub fn new() -> Self {
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

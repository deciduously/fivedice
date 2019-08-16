use rand::prelude::*;

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

struct Player {
    score: Score,
}

impl Player {
    fn new() -> Self {
        Self {
            score: Score::new(),
        }
    }
}

#[derive(Debug)]
enum RollResult {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
}

fn roll_die(mut rng: ThreadRng) -> RollResult {
    let idx = rng.gen_range(1, 7);
    use RollResult::*;
    match idx {
        1 => One,
        2 => Two,
        3 => Three,
        4 => Four,
        5 => Five,
        6 => Six,
        _ => unreachable!(),
    }
}

fn main() {
    let player = Player::new();
    let thread_rng = thread_rng();
    let roll = roll_die(thread_rng);
    println!("YAHTZEE\n\nScore: {:?}\nRoll: {:?}", player.score, roll);
}

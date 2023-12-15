// Public library
use robotics_lib::world::tile::Content;

// Standard library
use std::ops::Range;

pub const ROCK_LOOKING_FOR: [Content; 1] = [Content::Rock(0)];
pub const COIN_LOOKING_FOR: [Content; 4] = [Content::Coin(0), Content::Rock(0), Content::Garbage(0), Content::Tree(0)];
pub const BANK_LOOKING_FOR: [Content; 1] = [Content::Bank(Range { start: 0, end: 0 })];

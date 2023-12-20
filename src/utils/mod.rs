// Tools
use searchtool_unwrap::SearchDirection;

// Public library
use robotics_lib::world::tile::Content;

// Standard library
use std::ops::Range;

pub const ROCK_LOOKING_FOR: [Content; 1] = [Content::Rock(0)];
pub const COIN_LOOKING_FOR: [Content; 5] = [Content::Coin(0), Content::Rock(0), Content::Garbage(0), Content::Tree(0), Content::Fish(0)];
pub const BANK_LOOKING_FOR: [Content; 1] = [Content::Bank(Range { start: 0, end: 0 })];
pub const DIRECTIONS: [SearchDirection; 4] = [SearchDirection::BottomLeft, SearchDirection::BottomRight, 
                                                SearchDirection::TopLeft, SearchDirection::TopRight];

pub fn clone_direction(direction: &SearchDirection) -> SearchDirection {
    match direction {
        | SearchDirection::BottomLeft => SearchDirection::BottomLeft,
        | SearchDirection::BottomRight => SearchDirection::BottomRight,
        | SearchDirection::TopLeft => SearchDirection::TopLeft,
        | SearchDirection::TopRight => SearchDirection::TopRight,
    }
}
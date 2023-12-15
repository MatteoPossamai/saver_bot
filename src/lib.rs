mod utils;

// Tools
use charting_tools::ChartingTools; 
use charting_tools::charted_coordinate::ChartedCoordinate;
use recycle_by_ifrustrati::tool::recycle;
use charting_tools::charted_map::ChartedMap;
use arrusticini_destroy_zone::DestroyZone;

// Public library
use robotics_lib::runner::{Robot, Runnable};
use robotics_lib::world::World; 
use robotics_lib::world::coordinates::Coordinate;
use robotics_lib::event::events::Event;
use robotics_lib::runner::backpack::BackPack;
use robotics_lib::energy::Energy;
use robotics_lib::interface::{where_am_i, go, Direction, put};
use robotics_lib::world::tile::Content;

// Standard library
use std::fmt::Debug;
use std::ops::Range;

use crate::utils::{COIN_LOOKING_FOR, ROCK_LOOKING_FOR, BANK_LOOKING_FOR};

/// Represenst the state of the bot
/// - Collecting: The bot is collecting phase
/// - Connecting: The bot creating connections between banks
/// - Saving: The bot is saving the resources to banks
/// - Enjoying: The bot is enjoying the resources he collected
/// 
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    CoinCollecting,
    RockCollecting,
    Trading,
    Connecting,
    Saving,
    Enjoying,
    BankSearching
}

/// The SaverBot struct
/// It has a Robot field, so it can be used as a robot
/// It has a State field, so it can be used as a state machine
/// It has a Option<usize> field, so it can be used as a goal
/// 
/// # Examples
/// ```
/// use saver_bot::new_saver_bot;
/// use saver_bot::{SaverBot, State};
/// 
/// use robotics_lib::runner::{Robot, Runnable};
/// 
/// fn main () {
///   let  mut bot = new_saver_bot!(1);
///   println!("{:?}", bot);
/// }
pub struct SaverBot{
    pub robot: Robot,
    pub state: State,
    pub goal: Option<usize>,
    pub filled_banks: ChartedMap<Content>,
    pub unconnected_banks: ChartedMap<Content>,
    pub free_banks: ChartedMap<Content>,
    pub saved: usize
}

/// Initialized a new SaverBot, and you can ask for a goal
/// 
/// # Examples
/// ```
/// use saver_bot::new_saver_bot;
/// use saver_bot::{SaverBot, State};
/// 
/// use robotics_lib::runner::{Robot, Runnable};
/// 
/// fn main () {
///    let  mut bot = new_saver_bot!(1);
///    println!("{:?}", bot);
/// }
/// ```
#[macro_export]
macro_rules! new_saver_bot {
    () => {
        SaverBot{
            robot: Robot::new(),
            state: State::CoinCollecting,
            goal: None,
            filled_banks: ChartingTools::tool::<ChartedMap<Content>>().unwrap(), 
            unconnected_banks: ChartingTools::tool::<ChartedMap<Content>>().unwrap(),
            free_banks: ChartingTools::tool::<ChartedMap<Content>>().unwrap(),
            saved: 0
        }
    };
    ($x:expr) => {
        SaverBot{
            robot: Robot::new(),
            state: State::CoinCollecting,
            goal: Some($x),
            filled_banks: ChartingTools::tool::<ChartedMap<Content>>().unwrap(), 
            unconnected_banks: ChartingTools::tool::<ChartedMap<Content>>().unwrap(),
            free_banks: ChartingTools::tool::<ChartedMap<Content>>().unwrap(),
            saved: 0
        }
    };
}

/// Implementation of the Runnable trait for the SaverBot, 
/// so it can interact with the world through the API
/// in the intended manner
/// 
/// # Examples
/// ```
/// use saver_bot::new_saver_bot;
/// use saver_bot::{SaverBot, State};
/// 
/// use robotics_lib::runner::{Robot, Runnable};
/// 
/// fn main () {
///   // ...
///   let  mut bot = new_saver_bot!(1);
///   bot.process_tick(&mut world);
/// }
impl Runnable for SaverBot {
    fn process_tick(&mut self, world: &mut World) {
        // Here call all the utility functions, as a middleware
        // - look_for_banks

        // If enery to low, wait for recharge
        if !self.get_energy().has_enough_energy(15)  {
            return;
        }  

        // Check if the goal has been reached
        if let Some(goal) = self.goal {
            if self.get_coin_saved() >= goal {
                self.set_state(State::Enjoying);
            }
        }

        match self.get_state() {
            State::CoinCollecting => {
                self.coin_collect(world);
            }, 
            State::RockCollecting => {
                self.rock_collect(world);
            },
            State::Connecting => {
                self.connect(world);
            },
            State::Saving => {
                self.save(world);
            },
            State::Enjoying => {
                self.enjoy();
            },
            State::Trading => {
                self.trade();
            }, 
            State::BankSearching => {
                self.search_for_bank(world);
            }
        }
    }

    fn handle_event(&mut self, event: Event) {
        println!("{:?}", event);
        println!();
    }
    fn get_energy(&self) -> &Energy {
        &self.robot.energy
    }
    fn get_energy_mut(&mut self) -> &mut Energy {
        &mut self.robot.energy
    }
    fn get_backpack(&self) -> &BackPack {
        &self.robot.backpack
    }
    fn get_backpack_mut(&mut self) -> &mut BackPack {
        &mut self.robot.backpack
    }
    fn get_coordinate(&self) -> &Coordinate {
        &self.robot.coordinate
    }
    fn get_coordinate_mut(&mut self) -> &mut Coordinate {
        &mut self.robot.coordinate
    }
}

/// Implementation of Debug for development purposes
/// 
/// # Examples 
/// ```
/// let bot = new_saver_bot!(1);
/// println!("{:?}", bot);
/// ```
impl Debug for SaverBot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "State: {:?}, Goal: {:?}", self.state, self.goal)
    }
}

/// Utility functions to allow abstact some logic
/// and implement the state machine
impl SaverBot {
    pub fn new(goal: Option<usize>) -> Self {
        SaverBot{
            robot: Robot::new(),
            state: State::CoinCollecting,
            goal,
            filled_banks: ChartingTools::tool::<ChartedMap<Content>>().unwrap(), 
            unconnected_banks: ChartingTools::tool::<ChartedMap<Content>>().unwrap(),
            free_banks: ChartingTools::tool::<ChartedMap<Content>>().unwrap(),
            saved: 0
        }        
    }
    fn set_state(&mut self, state: State) {
        self.state = state;
    }
    fn get_state(&self) -> &State {
        &self.state
    }
    fn get_coin_saved(&self) -> usize {
        self.saved.clone()
    }
    fn wander_in_seach_of(&mut self, world: &mut World, contents: Vec<Content>) {
        // Save banks
        self.look_for_banks(world);

        // Get surrounding interesting content
        for content in contents {
            match DestroyZone.execute(world, self, content) {
                Ok((d, t)) => print!("destroyed {} on a total of {} objects", d, t),
                Err(e) => println!("Error: {:?}", e)
            }
        }

        // Look if something interesting nearby with the tool 
        // TODO: use the simple search tool and then move

        // temp
        let _ = go(self, world, Direction::Left);
        
    }
    fn look_for_banks(&mut self, world: &mut World) -> bool {
        let (neighborhoods, (x, y)) = where_am_i(self, &world);
        let mut found = false;

        // Searching if nearby there is a bank in the range
        for i in 0..3 {
            for j in 0..3 {
                let tile = &neighborhoods[i][j];
                if let Some(tile) = tile {
                    match &tile.content.to_default() {
                        Content::Bank(_) => {
                            found = true;
                            self.free_banks.save(&tile.content.to_default(), &ChartedCoordinate(x + i - 1, y + j - 1));
                        }
                        _ => {}
                    }
                }
                
            }
        }
        found
    }   
    fn coin_collect(&mut self, world: &mut World) {
        println!("Coin collecting");
        self.wander_in_seach_of(world, COIN_LOOKING_FOR.to_vec());
        
        let current_number_coins = self.get_backpack().get_contents().get(&Content::Coin(0)).unwrap();
        let current_number_garbage = self.get_backpack().get_contents().get(&Content::Garbage(0)).unwrap();
        let current_number_rock = self.get_backpack().get_contents().get(&Content::Rock(0)).unwrap();
        let current_number_trees = self.get_backpack().get_contents().get(&Content::Tree(0)).unwrap();

        // Change state if too many coin to save or if there are enough to trade
        if current_number_coins >= &12 {
            self.set_state(State::Saving)
        }else if (current_number_garbage >= &5) || (current_number_rock >= &3) || (current_number_trees >= &1) {
            self.set_state(State::Trading)
        }
    }
    fn rock_collect(&mut self, world: &mut World) {
        println!("Rock collecting");
        self.wander_in_seach_of(world, ROCK_LOOKING_FOR.to_vec());
        let current_number_rock = self.get_backpack().get_contents().get(&Content::Rock(0)).unwrap();

        // Change state if enough rock
        if current_number_rock >= &15 {
            self.set_state(State::Connecting)
        }
    }
    fn connect(&mut self, _world: &mut World) {
        println!("Connecting");
    }
    fn save(&mut self, world: &mut World) {
        println!("Saving");
        let direction = self.go_to_closest_open_bank(world);
        if let Some(dir) = direction {
            let putting = put(self, world, Content::Coin(0), 20, dir);
            match putting {
                Ok(quantity) => {
                    self.saved += quantity;
                    println!("Saved {quantity} coins");
                },
                Err(error) => println!("While saving there has been an issue {:?}", error)
            }
        } else if self.get_backpack().get_contents().get(&Content::Coin(0)).unwrap() <= &3 {
            self.set_state(State::CoinCollecting);
        } else {
            self.set_state(State::BankSearching);
        }
    }
    fn search_for_bank(&mut self, world: &mut World) {
        println!("Searching for bank");
        let found = self.look_for_banks(world);
        if found {
            self.set_state(State::Saving);
        } else {
            self.wander_in_seach_of(world, BANK_LOOKING_FOR.to_vec());
        }
    }
    fn enjoy(&mut self) {
        println!("Enjoying");
    }
    fn trade(&mut self) {
        // Call the recycle interface 
        let trade = recycle(self, 0);
        match trade {
            Ok(coins) => println!("You traded {} coins", coins),
            Err(error) => println!("While trading there has been an issue {:?}", error)
        }

        let current_number_coins = self.get_backpack().get_contents().get(&Content::Coin(0)).unwrap();
        if current_number_coins >= &12 {
            self.set_state(State::Saving)
        }else {
            self.set_state(State::CoinCollecting)
        }
    }
    fn go_to_closest_open_bank(&mut self, world: &mut World) -> Option<Direction> {
        let know_bank = self.free_banks.iter().len() > 0;
        if know_bank {
            let (x, y) = self.closest_bank();
            self.reach_position(world, x, y);
        } else {
            self.wander_in_seach_of(world, BANK_LOOKING_FOR.to_vec());
        }

        let (neighborhoods, (rx, ry)) = where_am_i(self, &world);
        for x in 0..3 {
            for y in 0..3 {
                let tile = &neighborhoods[x][y];
                if let Some(tile) = tile {
                    match &tile.content.to_default() {
                        Content::Bank(_) => {
                            let dir = if rx + 1 == x {Direction::Up} else if rx - 1 == x {Direction::Down} else if ry + 1 == y {Direction::Left} else {Direction::Right};
                            return Some(dir);
                        }
                        _ => {}
                    }
                }
            }
        }
        None
    }
    fn closest_bank(&mut self) -> (usize, usize) {
        let mut closest = (0, 0);
        let mut distance = 1000;
        if let Some(bank) = self.free_banks.get(&Content::Bank(Range{start: 0, end: 0})) {
            for (coord, _) in bank.iter() {
                let dist = (coord.0.pow(2) + coord.1.pow(2)) as usize;
                if dist < distance {
                    distance = dist;
                    closest = (coord.0, coord.1);
                }
            }
        }
        closest
    }
    fn reach_position(&mut self, world: &mut World, x: usize, y: usize) {
        while self.get_coordinate().get_row() < x {
            let _ = go(self, world, Direction::Down);
        }
        while self.get_coordinate().get_row() > x {
            let _ = go(self, world, Direction::Up);
        }
        while self.get_coordinate().get_col() < y {
            let _ = go(self, world,  Direction::Right);
        }
        while self.get_coordinate().get_col() > y {
            let _ = go(self, world, Direction::Left);
        }
    }
}


// robot_view and where_am_i to get the robot surroundings
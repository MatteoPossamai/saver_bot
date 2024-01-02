pub mod utils;

// Tools
use charting_tools::ChartingTools; 
use charting_tools::charted_coordinate::ChartedCoordinate;
use charting_tools::charted_map::ChartedMap;
use oxagaudiotool::OxAgAudioTool;
use oxagaudiotool::sound_config::OxAgSoundConfig;
use recycle_by_ifrustrati::tool::recycle;
use arrusticini_destroy_zone::DestroyZone;
use asfalt_inator::{Asphaltinator, Shape};
use searchtool_unwrap::SearchTool;
use holy_crab_best_path::shortest_path; // ?

// Public library
use robotics_lib::runner::{Robot, Runnable};
use robotics_lib::world::World; 
use robotics_lib::world::coordinates::Coordinate;
use robotics_lib::event::events::Event;
use robotics_lib::runner::backpack::BackPack;
use robotics_lib::energy::Energy;
use robotics_lib::interface::{where_am_i, go, Direction, put, destroy};
use robotics_lib::world::environmental_conditions::WeatherType;
use robotics_lib::world::tile::{Content, TileType};
use utils::clone_direction;

// Standard library
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Range;
use std::collections::BinaryHeap;
use rand::Rng;

use crate::utils::{COIN_LOOKING_FOR, ROCK_LOOKING_FOR, BANK_LOOKING_FOR, DIRECTIONS};

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

    // All the banks that the bot knows
    pub filled_banks: ChartedMap<Content>,
    pub unconnected_banks: ChartedMap<Content>,
    pub free_banks: ChartedMap<Content>,
    // pub connected_banks: ChartedMap<Content>,

    // Coins taken so far
    pub saved: usize,

    // Utility variables
    pub looking_for: Vec<Content>,
    pub audio: OxAgAudioTool,
    pub search_tool: SearchTool,
    pub timer: usize,

    
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
            // connected_banks: ChartingTools::tool::<ChartedMap<Content>>().unwrap(),
            free_banks: ChartingTools::tool::<ChartedMap<Content>>().unwrap(),
            saved: 0,
            looking_for: COIN_LOOKING_FOR.to_vec(),
            audio: SaverBot::audio_init(), 
            search_tool: SearchTool::new(),
            timer: 0
        }
    };
    ($x:expr) => {
        SaverBot{
            robot: Robot::new(),
            state: State::CoinCollecting,
            goal: Some($x),
            filled_banks: ChartingTools::tool::<ChartedMap<Content>>().unwrap(), 
            unconnected_banks: ChartingTools::tool::<ChartedMap<Content>>().unwrap(),
            // connected_banks: ChartingTools::tool::<ChartedMap<Content>>().unwrap(),
            free_banks: ChartingTools::tool::<ChartedMap<Content>>().unwrap(),
            saved: 0,
            looking_for: COIN_LOOKING_FOR.to_vec(),
            audio: SaverBot::audio_init(),
            search_tool: SearchTool::new(),
            timer: 0
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
        // Debug print
        println!("ROBOT");
        println!("- STATE: {:?}", self.state);
        println!("- POSITION: {:?}", self.robot.coordinate);
        println!("- ENERGY: {:?}", self.robot.energy.get_energy_level());
        println!("- BACKPACK: {:?}", self.robot.backpack);
        println!("- SAVED: {:?}", self.saved);
        println!("- KNOWN BANKS: {:?}", self.free_banks.iter());

        // Utility functions, to do all the things that can be done 
        // at the same time, regardless of what the robot is currently trying to do
        self.look_for_unknown_banks(world); // 0 energy required
        self.destroy_area(world); // Pay just if destroy something currently useful

        // If enery to low, wait for recharge
        if !self.get_energy().has_enough_energy(150)  {
            return;
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
        // let _ = self.audio.play_audio_based_on_event(&event); TODO: uncomment this for audio
        println!("{:?}", event);
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
            // connected_banks: ChartingTools::tool::<ChartedMap<Content>>().unwrap(),
            free_banks: ChartingTools::tool::<ChartedMap<Content>>().unwrap(),
            saved: 0,
            looking_for: COIN_LOOKING_FOR.to_vec(),
            audio: SaverBot::audio_init(),
            search_tool: SearchTool::new(),
            timer: 0
        }        
    }
    fn set_state(&mut self, state: State) {
        self.state = state;
    }
    fn get_state(&self) -> &State {
        &self.state
    }

    fn reach_position(&mut self, world: &mut World, x: usize, y: usize) -> bool {
        // TODO: Ask how to use the best path thing
        while self.get_coordinate().get_row() < x && self.get_energy().has_enough_energy(50) {
            let _ = go(self, world, Direction::Down);
        }
        while self.get_coordinate().get_row() > x && self.get_energy().has_enough_energy(50) {
            let _ = go(self, world, Direction::Up);
        }
        while self.get_coordinate().get_col() < y && self.get_energy().has_enough_energy(50){
            let _ = go(self, world,  Direction::Right);
        }
        while self.get_coordinate().get_col() > y && self.get_energy().has_enough_energy(50){
            let _ = go(self, world, Direction::Left);
        }

        self.get_coordinate().get_row() == x && self.get_coordinate().get_col() == y
    }
    fn connect(&mut self, _world: &mut World) {
        println!("Connecting");
        
        // if let Some(connected) = self.connected_banks.get(&Content::Bank(Range { start: 0, end: 0})) {
        //     if connected.len() > 0 {
        //         // remove the first from the free banks and put it in the connected banks
        //         let (cx, cy) = self.closest_bank();
        //         let _ = self.free_banks.remove(&Content::Bank(Range { start: 0, end: 0 }), ChartedCoordinate(cx, cy));
        //         self.connected_banks.save(&Content::Bank(Range { start: 0, end: 0 }), &ChartedCoordinate(cx, cy));
        //     }
        // }
        if self.get_backpack().get_contents().get(&Content::Rock(0)) < Some(&3) {
            self.set_state(State::RockCollecting);
        }
    }
    fn _connect_banks(&mut self, world: &mut World, x1: usize, y1: usize, x2: usize, _y2: usize) {
        // TODO, figure out something about the unfinished projects
        if self.reach_position(world, x1, y1) && self.get_energy().has_enough_energy(700) {
            let mut asphalitinator = Asphaltinator::new();
            let delta = x2 as isize - x1 as isize;
            let to_build = Shape::LongLong(delta.abs()as usize, if delta > 0 {Direction::Down} else {Direction::Up});
            let project = asphalitinator.design_project(to_build);
            match project {
                Ok(project) => {
                    let _ = asphalitinator.asfalting(self, world, project);
                },
                Err(error) => println!("While building there has been an issue {:?}", error)
            }
        }
        
        if self.get_backpack().get_contents().get(&Content::Rock(0)) < Some(&3) {
            self.set_state(State::RockCollecting);
        }
    }

    // -------------------
    // DONE CODE LOGIC
    // -------------------

    pub fn audio_init() -> OxAgAudioTool {
        // Configure events
        let mut events = HashMap::new();
        events.insert(Event::Ready, OxAgSoundConfig::new("assets/default/event/event_ready.ogg"));
        for i in 0..15 {
            events.insert(Event::AddedToBackpack(Content::Coin(0), i), OxAgSoundConfig::new("assets/default/event/event_add_to_backpack.ogg"));
            events.insert(Event::AddedToBackpack(Content::Rock(0), i), OxAgSoundConfig::new("assets/default/event/event_add_to_backpack.ogg"));
            events.insert(Event::AddedToBackpack(Content::Garbage(0), i), OxAgSoundConfig::new("assets/default/event/event_add_to_backpack.ogg"));
            events.insert(Event::AddedToBackpack(Content::Tree(0), i), OxAgSoundConfig::new("assets/default/event/event_add_to_backpack.ogg"));
        }
        events.insert(Event::EnergyRecharged(10), OxAgSoundConfig::new("assets/default/event/event_energy_recharged.ogg"));
        events.insert(Event::Terminated, OxAgSoundConfig::new("assets/default/event/event_terminated.ogg"));

        // Configure tiles
        let mut tiles = HashMap::new();
        tiles.insert(TileType::DeepWater, OxAgSoundConfig::new("assets/default/tile/tile_water.ogg"));
        tiles.insert(TileType::ShallowWater, OxAgSoundConfig::new("assets/default/tile/tile_water.ogg"));
        tiles.insert(TileType::Sand, OxAgSoundConfig::new("assets/default/tile/tile_sand.ogg"));
        tiles.insert(TileType::Grass, OxAgSoundConfig::new("assets/default/tile/tile_grass.ogg"));
        tiles.insert(TileType::Hill, OxAgSoundConfig::new("assets/default/tile/tile_grass.ogg"));
        tiles.insert(TileType::Mountain, OxAgSoundConfig::new("assets/default/tile/tile_mountain.ogg"));
        tiles.insert(TileType::Snow, OxAgSoundConfig::new("assets/default/tile/tile_snow.ogg"));
        tiles.insert(TileType::Lava, OxAgSoundConfig::new("assets/default/tile/tile_lava.ogg"));
        tiles.insert(TileType::Teleport(false), OxAgSoundConfig::new("assets/default/tile/tile_teleport.ogg"));
        tiles.insert(TileType::Street, OxAgSoundConfig::new("assets/default/tile/tile_street.ogg"));

        // Configure weather
        let mut weather = HashMap::new();
        weather.insert(WeatherType::Rainy, OxAgSoundConfig::new_looped_with_volume("assets/default/weather/weather_rainy.ogg", 0.4));
        weather.insert(WeatherType::Sunny, OxAgSoundConfig::new_looped("assets/default/weather/weather_sunny.ogg"));

        // Initialize audio
        let audio = OxAgAudioTool::new(events, tiles, weather);
        match audio {
            Ok(audio) => audio,
            Err(error) => panic!("Error while initializing audio: {:?}", error)
        }
        
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
    fn coin_collect(&mut self, world: &mut World) {
        println!("Coin collecting");
        if self.goal.is_some() && self.goal.unwrap() <= self.saved + self.get_backpack().get_contents().get(&Content::Coin(0)).unwrap() {
            self.set_state(State::Saving);
            return;
        }
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
    fn destroy_area(&mut self, world: &mut World) {
        let mut banks_points = vec![];
        if let Some(banks) = self.free_banks.get(&Content::Bank(Range { start: 0, end:0 })) {
            for bank in banks.iter() {
                banks_points.push((bank.0.0, bank.0.1));
            }
        }
        if let Some(banks) = self.unconnected_banks.get(&Content::Bank(Range { start: 0, end:0 })) {
            for bank in banks.iter() {
                banks_points.push((bank.0.0, bank.0.1));
            }
        }
        if let Some(banks) = self.filled_banks.get(&Content::Bank(Range { start: 0, end:0 })) {
            for bank in banks.iter() {
                banks_points.push((bank.0.0, bank.0.1));
            }
        }
        let (x, y) = (self.get_coordinate().get_row() as i32, self.get_coordinate().get_col() as i32);
        let directs = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 0), (0, 1), (1, -1), (1, 0), (1, -1)];
        let mut good = true;
        for (ox, oy) in directs.iter() {
            let (big_x, big_y) = (x + ox, y + oy);
            if banks_points.contains(&(big_x as usize, big_y as usize)) {
                good = false;
            }
        }
        if good {
            let needs = self.looking_for.clone();
            for content in needs.iter() {
                let _ = DestroyZone.execute(world, self, content.clone());
            }
        } else {
            let (tiles, _) = where_am_i(self, world);
            for i in 0..3 {
                for j in 0..3 {
                    let (cx, cy) = (x + i - 1, y + j - 1);
                    let tile = &tiles[i as usize][j as usize];
                    match tile {
                        None => {},
                        Some(tile) => {
                            let content = tile.content.clone();
                            if self.looking_for.contains(&content) && content != Content::Bank(Range { start: 0, end: 0 }) {
                                let direction = if cx > x {Direction::Down} else if cx < x {Direction::Up} else if cy > y {Direction::Right} else {Direction::Left};
                                let thing = destroy(self, world, direction);
                                match thing {
                                    Ok(number) => {println!("Destroyed {} {:?}", number, content);},
                                    Err(error) => println!("While destroying there has been an issue {:?}", error)
                                }
                            }
                        }
                    }
                }
            }

        }
        
    }
    fn rock_collect(&mut self, world: &mut World) {
        println!("Rock collecting");
        // remove all coins from the backpack
        let _ = put(self, world, Content::Coin(0), self.get_backpack().get_contents().get(&Content::Coin(0)).unwrap().clone(), Direction::Up);
        // remove all garbage from the backpack
        let _ = put(self, world, Content::Garbage(0), self.get_backpack().get_contents().get(&Content::Garbage(0)).unwrap().clone(), Direction::Up);
        self.wander_in_seach_of(world, ROCK_LOOKING_FOR.to_vec());
        let current_number_rock = self.get_backpack().get_contents().get(&Content::Rock(0)).unwrap();
        println!("CURRENT number of rock: {:?}", current_number_rock);
        // Change state if enough rock
        if current_number_rock >= &15 {
            println!("Changing state to connecting");
            self.set_state(State::Connecting)
        }
    }
    fn enjoy(&mut self) {
        println!("Enjoying");
        // TODO: Add maybe some useless celebrations and stuff, not meaningful now
    }
    fn search_for_bank(&mut self, world: &mut World) {
        println!("Searching for bank");
        if self.free_banks.get(&Content::Bank(Range { start: 0, end: 0 })).iter().len() > 0 {
            self.set_state(State::Saving);
        } else {
            self.look_for_unknown_banks(world);
            self.wander_in_seach_of(world, BANK_LOOKING_FOR.to_vec());
        }
    }
    fn go_to_closest_open_bank(&mut self, world: &mut World) -> Option<Direction> {
        let know_bank = self.free_banks.iter().len() > 0;
        if know_bank {
            let (x, y) = self.closest_bank();
            println!("Closest bank is at {:?} {:?}", x, y);
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
    fn look_for_unknown_banks(&mut self, world: &mut World) {
        let (neighborhoods, (x, y)) = where_am_i(self, &world);

        let current_banks = self.free_banks.iter();
        let mut seend_coord: Vec<(usize, usize)> = vec![];
        for (_, coord) in current_banks {
            for (pos, _) in coord {
                seend_coord.push((pos.0, pos.1));
            }
        }

        // Searching if nearby there is a bank in the range
        for i in 0..3 {
            for j in 0..3 {
                let tile = &neighborhoods[i][j];
                if let Some(tile) = tile {
                    match &tile.content.to_default() {
                        Content::Bank(_) => {
                            if !seend_coord.contains(&(x + i - 1, y + j - 1)) {
                                self.free_banks.save(&tile.content.to_default(), &ChartedCoordinate(x + i - 1, y + j - 1));
                            }
                        }
                        _ => {}
                    }
                }
                
            }
        }
    }
    fn wander_in_seach_of(&mut self, world: &mut World, contents: Vec<Content>) {
        self.destroy_area(world);

        // Look if something interesting nearby with the tool 
        let mut st = SearchTool::new();
        
        self.timer += 1;
        let res = st.look_for_this_content(self, world, contents.clone(),
                2 , clone_direction(&DIRECTIONS[rand::thread_rng().gen_range(0..4)]));
        match res {
            Ok(_) => {
                // Save the banks into the map
                if contents.contains(&Content::Bank(Range{start: 0, end: 0})) {
                    for (_, coord) in st.found_content_coords.iter() {
                        for (posx, posy) in coord {
                            if let Some(coord) = self.free_banks.clone().get(&Content::Bank(Range { start: 0, end: 0 })) {
                                for (coord, _) in coord {
                                    if coord.0 != posx.clone() || coord.1 != posy.clone() {
                                        self.free_banks.save(&Content::Bank(Range { start: 0, end: 0 }), &ChartedCoordinate(posx.clone(), posy.clone()));
                                    }
                                }
                            }
                        }
                    }
                }else {
                    let mut heap = BinaryHeap::new();
                    // Pupulate heap for closest stuff to current distance
                    let (x, y) = (self.get_coordinate().get_row(), self.get_coordinate().get_col());
                    for (_, coord) in st.found_content_coords.iter() {
                        for (posx, posy) in coord {
                            let dist = (posx.clone() as isize - x as isize).abs() + (posy.clone() as isize - y as isize).abs();
                            heap.push((dist, (posx.clone(), posy.clone())));
                        }
                    }

                    while self.get_energy().has_enough_energy(400) && heap.len() > 0 {
                        let (_, (x, y)) = heap.pop().unwrap();
                        let _ = self.reach_position(world, x, y);
                        self.destroy_area(world);
                    }
                }
            },
            Err(e) => println!("Error: {:?}", e)
        }
        for _ in 0..4 {
            let _ = go(self, world, [Direction::Up, Direction::Down, Direction::Left, Direction::Right][rand::thread_rng().gen_range(0..4)].clone());
        }
        
    }
    fn closest_bank(&mut self) -> (usize, usize) {
        let mut closest = (0, 0);
        let mut distance = 1000;
        let robot_x = self.get_coordinate().get_row();
        let robot_y = self.get_coordinate().get_col();

        if let Some(bank) = self.free_banks.get(&Content::Bank(Range{start: 0, end: 0})) {
            for (coord, _) in bank.iter() {

                let dist = (coord.0 as isize - robot_x as isize).abs() + (coord.1 as isize - robot_y as isize).abs();

                if dist < distance {
                    distance = dist;
                    closest = (coord.0, coord.1);
                }
            }
        }
        closest
    }
    fn save(&mut self, world: &mut World) {
        println!("Saving");
        let (cx, cy) = self.closest_bank();
        let (x, y) = (self.get_coordinate().get_row(), self.get_coordinate().get_col());
         
        let mut direction = self.go_to_closest_open_bank(world);

        if (cx == x) && (cy == y) {
            let res = go(self, world, Direction::Left);
            match res {
                Ok(_) => {direction = Some(Direction::Left);},
                Err(_) => {
                    let res = go(self, world, Direction::Right); 
                    
                    match res {Ok(_) => {direction = Some(Direction::Right);}, Err(_) => {
                        let res = go(self, world, Direction::Up);
                        
                        match res {Ok(_) => {direction = Some(Direction::Up);}, Err(_) => {
                            let _ = go(self, world, Direction::Down);
                            direction = Some(Direction::Down);
                        }}
                    }}}
            }
        }
        if let Some(dir) = direction {
            let putting = put(self, world, Content::Coin(0), self.get_backpack().get_contents().get(&Content::Coin(0)).unwrap().clone(), dir);
            match putting {
                Ok(quantity) => {
                    if quantity == 0 {
                        // remove current bank from the free banks and put it in the filled banks
                        let _ = self.free_banks.remove(&Content::Bank(Range { start: 0, end: 0 }), ChartedCoordinate(cx, cy));
                        self.filled_banks.save(&Content::Bank(Range { start: 0, end: 0 }), &ChartedCoordinate(cx, cy));
                    }
                    self.saved += quantity;
                    println!("Saved {quantity} coins");
                    if let Some(goal) = self.goal {
                        if self.saved >= goal {
                            self.set_state(State::RockCollecting);
                        }else {
                            self.set_state(State::CoinCollecting);  
                        }
                    }else {
                        self.set_state(State::CoinCollecting);
                    }
                },
                Err(error) => println!("While saving there has been an issue {:?}", error)
            }
        } else {
            // If the goal is met, then go to rock collect, 
            // otherwise search for banks
            if let Some(goal) = self.goal {
                if self.saved >= goal {
                    self.set_state(State::RockCollecting);
                }else {
                    self.set_state(State::BankSearching);
                }
            }else {
                self.set_state(State::BankSearching);
            }
        }
    }
}
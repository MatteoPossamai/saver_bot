use std::collections::HashMap;
use std::fmt::Debug;

use oxagaudiotool::sound_config::OxAgSoundConfig;
use robotics_lib::runner::{Robot, Runnable};
use robotics_lib::world::World; 
use robotics_lib::world::coordinates::Coordinate;
use robotics_lib::event::events::Event;
use robotics_lib::runner::backpack::BackPack;
use robotics_lib::energy::Energy;

use oxagaudiotool::audio_tool::OxAgAudioTool;
use robotics_lib::world::environmental_conditions::WeatherType;
use robotics_lib::world::tile::{Content, TileType};
use recycle_by_ifrustrati::tool::recycle; // Top utility for this bot
use charting_tools;

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
    Connecting,
    Saving,
    Enjoying,
}

/// Represents the mood of the bot
/// - Stressed: The bot is stressed
/// - Calm: The bot is calm
/// - Happy: The bot is happy
/// - Euphoric: The bot is euphoric
#[derive(Debug)]
pub enum Mood {
    Stressed, 
    Calm, 
    Happy,
    Euphoric
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
    pub mood: Mood,
    pub unconnected_banks: HashMap<usize, Coordinate>,
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
            mood: Mood::Stressed,
            unconnected_banks: HashMap::new()
            // sound: generate_sound_tool(),
        }
    };
    ($x:expr) => {
        SaverBot{
            robot: Robot::new(),
            state: State::CoinCollecting,
            goal: Some($x),
            mood: Mood::Stressed,
            unconnected_banks: HashMap::new()
            // sound: generate_sound_tool(),
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
    fn process_tick(&mut self, _world: &mut World) {
        println!("IDLE");
    }

    fn handle_event(&mut self, event: Event) {
        println!();
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
        write!(f, "State: {:?}, Goal: {:?}, Mood: {:?}", self.state, self.goal, self.mood)
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
            mood: Mood::Stressed,
            unconnected_banks: HashMap::new()
        }        
    }
}

/// Utility functions to allow abstact logic of the creation of an stressed sound
pub fn generate_sound_mood_1() -> OxAgAudioTool {
    todo!()
}

/// Utility functions to allow abstact logic of the creation of an calm sound
pub fn generate_sound_mood_2() -> OxAgAudioTool {
    todo!()
}

/// Utility functions to allow abstact logic of the creation of an happy sound
pub fn generate_sound_mood_3() -> OxAgAudioTool {
    todo!()
}

/// Utility functions to allow abstact logic of the creation of an euphoric sound
pub fn generate_sound_mood_4() -> OxAgAudioTool {
    todo!()
}
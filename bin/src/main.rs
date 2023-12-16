// My AI
use saver_bot::new_saver_bot;
use saver_bot::{SaverBot, State};
use saver_bot::utils::COIN_LOOKING_FOR;

// Tools
use charting_tools::ChartingTools;
use oxagaudiotool::error::error::OxAgAudioToolError;
use charting_tools::charted_map::ChartedMap;
use worldgen_unwrap::public::WorldgeneratorUnwrap;
use oxagaudiotool::sound_config::OxAgSoundConfig;

// Public library
use robotics_lib::world::tile::Content;
use robotics_lib::runner::{Robot, Runner};

// Standard library
use std::thread::sleep;
use std::time::Duration;

fn main ()  -> Result<(), OxAgAudioToolError> {
    println!("Loading...");
    let background_music = OxAgSoundConfig::new_looped_with_volume("assets/default/music.ogg", 2.0);
    
    // Robot and world initialization
    let mut bot = new_saver_bot!();
    let mut world_gen = WorldgeneratorUnwrap::init(false, None);
    bot.audio.play_audio(&background_music)?;
    let run = Runner::new(Box::new(bot), &mut world_gen);

    match run {
        | Ok(mut r) => {
            let _ = loop {
                let _ = r.game_tick();
                sleep(Duration::from_millis(500));
                println!("{} {}", r.get_robot().get_coordinate().get_row(), r.get_robot().get_coordinate().get_col());
                println!("{:?}", r.get_robot().get_backpack());
            };
        }
        | Err(e) => println!("{:?}", e),
    }

    Ok(())
}
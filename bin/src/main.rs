use oxagaudiotool::error::error::OxAgAudioToolError;
use oxagaudiotool::sound_config::OxAgSoundConfig;
use saver_bot::new_saver_bot;
use saver_bot::{SaverBot, State, Mood};

use robotics_lib::runner::{Robot, Runnable};
use std::collections::HashMap;


fn main ()  -> Result<(), OxAgAudioToolError> {
    let  mut bot = new_saver_bot!(1);
    println!("{:?}", bot);



    Ok(())
}

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use toml::value::Datetime;
use serde_derive::Deserialize;

use thiserror::Error;

const DEFAULT_SHARE : &str = "/.local/share/Septem/";
const DEFAULT_CONFIG : &str = "/.config/Septem/septem.toml";

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to retrieve env HOME:\n{0}")]
    EnvError(#[from] env::VarError),
    
    #[error("Failed to find or read config file:\n{0}")]
    FileIOError(#[from] io::Error),

    #[error("Toml-rs failed to parse the config file:\n{0}")]
    TomlError(#[from] toml::de::Error),
    
    #[error("You're Still Here? It's Over, Go Home.")]
    UnknownError,
}

#[derive(Deserialize, Debug)]
pub struct EventHandlerConfig {
    min_focus_time : u8,
}

impl Default for EventHandlerConfig {
    fn default() -> Self {
        Self { min_focus_time : 10 }
    }
}

#[derive(Deserialize, Debug)]
pub struct RecorderConfig {
    productive : Vec<String>,
    unproductive : Vec<String>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct MonthWeekDay {
    month : u8,
    week : u8, 
    day : u8,
}

#[derive(Clone, Deserialize, Debug)]
pub struct MonthDay {
    month : u8,
    day : u8,
}

#[derive(Clone, Deserialize, Debug)]
pub struct WeekDay {
    week : u8,
    day : u8,
}

#[derive(Clone, Deserialize, Debug, Default)]
pub struct DateTimeConfig {
    month_week_days : Vec<MonthWeekDay>,
    month_days : Vec<MonthDay>,
    week_days : Vec<WeekDay>,
    start_hour : u8,  // 24 hour time 
    stop_hour : u8,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    shared_dir : Option<String>,
    blacklist_times : Option<DateTimeConfig>,
    event_handler : Option<EventHandlerConfig>,
    recorder : RecorderConfig,
}

impl Config {

    // Temp default solution
    pub fn new() -> Result<Config, ConfigError> {
        let home = env::var("HOME")?;
        let mut config_file = String::new();
        
        File::open(home + DEFAULT_CONFIG)
            .and_then(|mut f| f.read_to_string(&mut config_file))?;
       
        let config : Config = toml::from_str(config_file.as_str())?; 
        Ok(config)
    }
       
    pub fn shared_dir(&self) -> Result<String, ConfigError> {
        let mut home_share = env::var("HOME")?;
        home_share += DEFAULT_SHARE;
        
        let share_dir = self.shared_dir.clone().unwrap_or(home_share);

        Ok(share_dir.to_owned())
    }
    
    pub fn blacklists(&self) -> DateTimeConfig {
        self.blacklist_times.clone().unwrap_or_default()
    }
    
/*
    pub fn proc_names<'a>(&'a self) -> &'a Vec<String> {
        &self.proc_names
    }

    pub fn dates<'a>(&'a self) -> &'a Vec<Datetime> {
        &self.dates
    }

    pub fn start_hour(&self) -> u8 {
        self.start_hour
    }

    pub fn stop_hour(&self) -> u8 {
        self.stop_hour
    }

    pub fn min_sec(&self) -> u8 {
        self.min_sec
    }
*/

}



#![allow(dead_code)]
#![allow(unused_variables)]

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

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

trait Month {
    fn month(&self) -> Option<u8>;
}

trait Week {
    fn week(&self) -> Option<u8>;
}

trait Day {
    fn day(&self) -> Option<u8>;
}

/* 
 * TODO:
 *
 * Alert System Configurations
 *
 *
 */

#[derive(Clone, Deserialize, Debug)]
pub struct EventHandlerConfig {
    min_focus_time : u8,
}

impl EventHandlerConfig {
    pub fn min_focus_time(&self) -> u8 {
        self.min_focus_time
    }
}

impl Default for EventHandlerConfig {
    fn default() -> Self {
        Self { min_focus_time : 10 }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct RecorderConfig {
    productive : Vec<String>,
    unproductive : Vec<String>,
}

impl<'a> RecorderConfig { 
    pub fn productive(&'a self) -> &'a Vec<String> {
        &self.productive
    }
    
    pub fn unproductive(&'a self) -> &'a Vec<String> {
        &self.unproductive
    }
}

#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum Date {
    MonthWeekDay{month : u8, week : u8, day : u8},
    MonthDay{month : u8, day : u8},
    WeekDay{week : u8, day : u8},
}

impl Month for Date {
    fn month(&self) -> Option<u8> {
        match *self {
            Self::MonthWeekDay{month, week : _, day : _} => Some(month),
            Self::MonthDay{month, day : _} => Some(month),
            Self::WeekDay{week : _, day : _} => None,
        }
    }
}

impl Week for Date {
    fn week(&self) -> Option<u8> {
        match *self {
            Self::MonthWeekDay{month : _, week, day : _} => Some(week),
            Self::MonthDay{month : _, day : _} => None,
            Self::WeekDay{week, day : _} => Some(week),
        }
    }
}

// MWD and WD are day of the week
// MD is day of the month
impl Day for Date {
    fn day(&self) -> Option<u8> {
        match *self {
            Self::MonthWeekDay{month : _, week : _, day} => Some(day),
            Self::MonthDay{month : _, day} => Some(day),
            Self::WeekDay{week : _, day} => Some(day),
        }
    }
}

#[derive(Clone, Deserialize, Debug, Default)]
pub struct DateTimeConfig {
    dates : Vec<Date>,
    start_hour : Option<u8>,  // 24 hour time 
    stop_hour : Option<u8>,
}

impl<'a> DateTimeConfig { 
    pub fn dates(&'a self) -> &'a Vec<Date> {
        &self.dates
    }

    pub fn start_hour(&self) -> u8 {
        self.start_hour.unwrap_or(0)
    }

    pub fn stop_hour(&self) -> u8 {
        self.stop_hour.unwrap_or(0)
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    shared_dir : Option<String>,
    blacklist : Option<DateTimeConfig>,
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
    
    pub fn blacklists_dates(&self) -> DateTimeConfig {
        self.blacklist.clone().unwrap_or_default()
    }
 
    pub fn event_handler_config(&self) -> EventHandlerConfig {
        self.event_handler.clone().unwrap_or_default()
    }

    pub fn recorder_config(&self) -> RecorderConfig {
        self.recorder.clone()
    }
}


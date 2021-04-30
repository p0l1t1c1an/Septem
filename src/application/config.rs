#![allow(dead_code)]
#![allow(unused_variables)]

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use serde_derive::Deserialize;

use thiserror::Error;

const DEFAULT_SHARE: &'static str = "/.local/share/Septem/";
const DEFAULT_CONFIG: &'static str = "/.config/Septem/septem.toml";

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to retrieve env HOME:\n{0}")]
    EnvError(#[from] env::VarError),

    #[error("Failed to find or read config file:\n{0}")]
    FileIOError(#[from] io::Error),

    #[error("Toml-rs failed to parse the config file:\n{0}")]
    TomlError(#[from] toml::de::Error),
}

#[derive(Clone, Deserialize, Debug)]
pub struct AlertConfig {
    delay: u64,
    trigger_time: u64,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            delay: 5, // 5 seconds deley
            trigger_time: 20, // 20 minute trigger
        }
    } 
}

impl AlertConfig {
    pub fn delay(&self) -> u64 {
        self.delay
    }

    pub fn trigger_time(&self) -> u64 {
        self.trigger_time
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct RecorderConfig {
    productive: Vec<String>,
}

impl<'a> RecorderConfig {
    pub fn productive(&'a self) -> &'a Vec<String> {
        &self.productive
    }
}

#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum Date {
    MonthWeekDay { month: u8, week: u8, day: u8 },
    MonthDay { month: u8, day: u8 },
    WeekDay { week: u8, day: u8 },
}

impl Date {
    fn month(&self) -> Option<u8> {
        match *self {
            Self::MonthWeekDay {
                month,
                week: _,
                day: _,
            } => Some(month),
            Self::MonthDay { month, day: _ } => Some(month),
            Self::WeekDay { week: _, day: _ } => None,
        }
    }

    fn week(&self) -> Option<u8> {
        match *self {
            Self::MonthWeekDay {
                month: _,
                week,
                day: _,
            } => Some(week),
            Self::MonthDay { month: _, day: _ } => None,
            Self::WeekDay { week, day: _ } => Some(week),
        }
    }

    // MWD and WD are day of the week
    // MD is day of the month
    fn day(&self) -> Option<u8> {
        match *self {
            Self::MonthWeekDay {
                month: _,
                week: _,
                day,
            } => Some(day),
            Self::MonthDay { month: _, day } => Some(day),
            Self::WeekDay { week: _, day } => Some(day),
        }
    }
}

#[derive(Clone, Deserialize, Debug, Default)]
pub struct DateTimeConfig {
    dates: Vec<Date>,
    start_hour: Option<u8>, // 24 hour time
    stop_hour: Option<u8>,
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
    shared_dir: Option<String>,
    blacklist: Option<DateTimeConfig>,
    recorder: RecorderConfig,
    alert: Option<AlertConfig>,
}

impl Config {
    // Temp default solution
    pub fn new(c: Option<String>) -> Result<Config, ConfigError> {
        let config_path = match c {
            Some(s) => s,
            None => env::var("HOME")? + DEFAULT_CONFIG,
        };

        let mut config_contents = String::new();

        File::open(config_path).and_then(|mut f| f.read_to_string(&mut config_contents))?;
        Ok(toml::from_str(config_contents.as_str())?)
    }

    pub fn shared_dir(&self) -> Result<String, ConfigError> {
        let share_dir = self.shared_dir.to_owned();
        match share_dir {
            Some(s) => Ok(s),
            None => Ok(env::var("HOME")? + DEFAULT_SHARE),
        }
    }

    pub fn blacklists_dates(&self) -> DateTimeConfig {
        self.blacklist.to_owned().unwrap_or_default()
    }

    pub fn recorder_config(&self) -> RecorderConfig {
        self.recorder.to_owned()
    }

    pub fn alert_config(&self) -> AlertConfig {
        self.alert.to_owned().unwrap_or_default()
    }
}

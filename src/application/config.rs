#![allow(dead_code)]
#![allow(unused_variables)]

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use chrono::Weekday;
use serde_derive::Deserialize;

use thiserror::Error;

const DEFAULT_SHARE: &str = "/.local/share/Septem/";
const DEFAULT_CONFIG: &str = "/.config/Septem/septem.toml";

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to retrieve env HOME:\n{0}")]
    EnvError(#[from] env::VarError),

    #[error("Failed to find or read config file:\n{0}")]
    FileIoError(#[from] io::Error),

    #[error("Toml-rs failed to parse the config file:\n{0}")]
    TomlError(#[from] toml::de::Error),
}

pub type Hours = (Weekday, u8, u8); 

// Todo: Add Enum and alert type for config
// It can be a pop up message or play audio
// Rn, I will just make it println! a message

#[derive(Clone, Deserialize, Debug)]
pub struct AlertConfig {
    productive_time: f64,
    unproductive_time: f64,
    message: String,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            productive_time: 5.0,    // Resets at 5 minutes
            unproductive_time: 20.0, // Prints message at 5 minutes
            message: "You have been wasting time.\nPlease start being productive.".to_owned(),
        }
    }
}

impl AlertConfig {
    pub fn productive_time(&self) -> f64 {
        self.productive_time
    }

    pub fn unproductive_time(&self) -> f64 {
        self.unproductive_time
    }

    pub fn message(&self) -> &String {
        &self.message
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct RecorderConfig {
    notify_delay: u64,
    write_delay: u64,
    productive: Vec<String>,
}

impl RecorderConfig {
    pub fn productive(&self) -> &Vec<String> {
        &self.productive
    }

    pub fn notify_delay(&self) -> u64 {
        self.notify_delay
    }

    pub fn write_delay(&self) -> u64 {
        self.write_delay
    }
}

#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum Date {
    MonthWeekDay { month: u8, week: u8, day: u8 },
    MonthDay { month: u8, day: u8 },
}

impl Date {
    pub fn month(&self) -> u8 {
        match *self {
            Self::MonthWeekDay {
                month,
                week: _,
                day: _,
            } => month,
            Self::MonthDay { month, day: _ } => month,
        }
    }

    pub fn week(&self) -> Option<u8> {
        match *self {
            Self::MonthWeekDay {
                month: _,
                week,
                day: _,
            } => Some(week),
            Self::MonthDay { month: _, day: _ } => None,
        }
    }

    // MWD are day of the week
    // MD is day of the month
    pub fn day(&self) -> u8 {
        match *self {
            Self::MonthWeekDay {
                month: _,
                week: _,
                day,
            } => day,
            Self::MonthDay { month: _, day } => day,
        }
    }
}

#[derive(Clone, Deserialize, Debug, Default)]
pub struct DateTimeConfig {
    disabled_days: Vec<Date>,
    start_hours: Vec<Hours>,
}

impl DateTimeConfig {
    pub fn dates(&self) -> &Vec<Date> {
        &self.disabled_days
    }

    pub fn start_hours(&self) -> &Vec<Hours> {
        &self.start_hours
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    share_directory: Option<String>,
    recorder: RecorderConfig,
    date_and_time: Option<DateTimeConfig>,
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

    pub fn share(&self) -> Result<String, ConfigError> {
        let share = self.share_directory.to_owned();
        match share {
            Some(s) => Ok(s),
            None => Ok(env::var("HOME")? + DEFAULT_SHARE),
        }
    }

    pub fn recorder_config(&self) -> RecorderConfig {
        self.recorder.to_owned()
    }
    
    pub fn date_config(&self) -> DateTimeConfig {
        self.date_and_time.to_owned().unwrap_or_default()
    }

    pub fn alert_config(&self) -> AlertConfig {
        self.alert.to_owned().unwrap_or_default()
    }
}

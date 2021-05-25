#![allow(dead_code)]
#![allow(unused_variables)]

pub mod alert_config;
pub mod date_config;
pub mod event_config;
pub mod recorder_config;

use alert_config::AlertConfig;
use date_config::DateTimeConfig;
use event_config::EventConfig;
use recorder_config::RecorderConfig;

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

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

#[derive(Deserialize, Debug)]
pub struct Config {
    share_directory: Option<String>,
    recorder: RecorderConfig,
    event_handler: Option<EventConfig>,
    date_and_time: Option<DateTimeConfig>,
    alerts: Option<AlertConfig>,
}

type BrokenUp = (String, RecorderConfig, EventConfig, DateTimeConfig, AlertConfig);

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

    pub fn event_config(&self) -> EventConfig {
        self.event_handler.to_owned().unwrap_or_default()
    }

    pub fn date_config(&self) -> DateTimeConfig {
        self.date_and_time.to_owned().unwrap_or_default()
    }

    pub fn alert_config(&self) -> AlertConfig {
        self.alerts.to_owned().unwrap_or_default()
    }

    pub fn break_up(self) -> Result<BrokenUp, ConfigError> {
        Ok((
            self.share()?,
            self.recorder_config(),
            self.event_config(),
            self.date_config(),
            self.alert_config(),
        ))
    }
}

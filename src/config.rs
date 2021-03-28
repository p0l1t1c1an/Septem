
use std::env;
use std::time::Duration;
use std::error::Error;

use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};

use chrono::{Date, Local};

use toml::DateTime;
use serde_derive::Deserialize;

const DEFAULT_SHARE : &str = "/.local/share/Septem/";
const DEFAULT_CONFIG : &str = "/.config/Septem/septem.toml";

pub enum ConfigError {
    G,
    T(toml::Error),
}


pub struct Config {
    shared_dir : String,
    process_names : Vec<String>,
    dates : Vec<Date<Local>>,
    start_hour : u8,
    stop_hour : u8,
    min_sec : u8,
}

impl Config {

    // Defaults will be overwritten when parsing config file
    // Will only set share directory if it is empty beforehand
    pub fn new(c : String) -> Result<Config, ConfigError> {
        // Take parsed config file and check that values are valid
    }
    
    pub fn new_with_share(c : String, s : String) -> Result<Config, ConfigError> {
        // Same as normal config then set shared directory
        // Will generate files in share if they don't exist already
    }

    fn parse(c : String) -> Result<Config, ConfigError> {
        // Use toml::from_str to read config file and generate
        // config or return error from toml
        // Will need to read file and load into string first
    }
    
    pub fn shared_dir<'a>(&'a self) -> &'a String {
        &self.shared_dir
    }
    
    pub fn whitelist<'a>(&'a self) -> &'a Vec<String> {
        &self.whitelist
    }
}

#[macro_export]
macro_rules! config {
    (config:$c:expr, share:$s:expr) => { // Both Strings
        Config::new_with_share($c, $s)
    }

    (config:$c:expr) => {
        Config::new($c)
    }

    (share:$s:expr) => {
        let home = env::var("HOME").unwrap();
        Config::new_with_share(home + DEFAULT_CONFIG, $s)
    }

    () => {
        let home = env::var("HOME").unwrap();
        Config::new(home + DEFAULT_CONFIG)
    }
}

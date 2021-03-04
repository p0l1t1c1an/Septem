
use std::env;
use std::time::Duration;
use std::error::Error;

use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};

const DEFAULT_SHARE : &str = "/.local/share/Septem/";
const DEFAULT_CONFIG : &str = "/.config/Septem/septem.toml";

pub enum ConfigError {
    G,
}

pub struct Config {
    config_file : String,
    shared_dir : String,
    whitelist : Vec<String>,
    start_hour : u8,
    stop_hour : u8,
    // TODO: Add vector of Dates to disable Septem on  
}

impl Config {
    pub fn new(c : String) -> Result<Config, ConfigError> {

    }
    
    pub fn new_with_share(c : String, s : String) -> Result<Config, ConfigError> {
        
    }
    
    pub fn config_file<'a>(&'a self) -> &'a String {
        &self.config_file
    }
    
    pub fn shared_dir<'a>(&'a self) -> &'a String {
        &self.shared_dir
    }
    
    pub fn whitelist<'a>(&'a self) -> &'a Vec<String> {
        &self.whitelist
    }
}

/*
 * TODO 
 * This needs an option where we can not provide a share directory
 * and then load a share option from the config.
 *
 */


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

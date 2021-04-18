
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
enum ConfigError {
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
pub struct Config {
    shared_dir : String,
    proc_names : Vec<String>,
    dates : Vec<Datetime>,
    start_hour : u8,
    stop_hour : u8,
    min_sec : u8,
}

impl Config {

    // Temp default solution
    pub fn new() -> Result<Config, ConfigError> {
        let home = env::var("HOME")?;
        let mut config_file = String::new();
        
        File::open(home + DEFAULT_CONFIG)
            .and_then(|mut f| f.read_to_string(config_file))?;
        
        let config : Config = toml::from_str(config_file)?; 
        Ok(config)
    }
       
    pub fn shared_dir<'a>(&'a self) -> &'a String {
        &self.shared_dir
    }
    
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
}


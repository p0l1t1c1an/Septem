
pub use self::config::*;

mod config {
    use std::env;
    use std::time::Duration;
    use std::error::Error;
    
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::{self, BufReader};

    pub enum ConfigError {
        G,
    }

    pub struct Config {
        config_file : String,
        shared_dir : String,
        delay : Duration,
        whitelist : Vec<String>,
        use_PID : bool,
        use_name : bool,
    }

    impl Config {
        pub fn new(c : String, s : String) -> Result<Config, ConfigError> {

        }

        pub fn config_file<'a>(&'a self) -> &'a String {
            &self.config_file
        }
        
        pub fn shared_dir<'a>(&'a self) -> &'a String {
            &self.shared_dir
        }

        pub fn delay(&self) -> Duration {
            self.delay
        }
        
        pub fn whitelist<'a>(&'a self) -> &'a Vec<String> {
            &self.whitelist
        }

        pub fn use_PID(&self) -> bool {
            self.use_PID
        }

        pub fn use_name(&self) -> bool {
            self.use_name
        }
    }

    #[macro_export]
    macro_rules! config {
        (config:$c:expr, share:$s:expr) => { // Both Strings
            Config::new($c, $s)
        }

        (config:$c:expr) => {
            let home = env::var("HOME").unwrap();
            Config::new($c, home + "/.local/share/Septem/")
        }
 
        (share:$s:expr) => {
            let home = env::var("HOME").unwrap();
            Config::new(home + "/.config/Septem/septem.toml", $s)
        }

        () => {
            let home = env::var("HOME").unwrap();
            Config::new(home + "/.config/Septem/septem.toml", home + "/.local/share/Septem/")
        }
    }
}

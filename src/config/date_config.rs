
use chrono::{Weekday, Month};
use serde_derive::Deserialize;

#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum Date {
    MonthWeekDay { month: Month, week: u32, day: Weekday },
    MonthDay { month: Month, day: u32 },
}

#[derive(Clone, Deserialize, Debug)]
pub struct Hours { 
    weekday: Weekday, 
    start: u32, 
    stop: u32 
}

impl Hours {
    pub fn weekday(&self) -> Weekday {
        self.weekday
    }

    pub fn start(&self) -> u32 {
        self.start
    }
    
    pub fn stop(&self) -> u32 {
        self.stop
    }
}


impl Default for Hours {
    fn default() -> Self {
        Self {
            weekday: Weekday::Mon,
            start: 0,
            stop: 24,
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



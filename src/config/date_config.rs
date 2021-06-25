use chrono::{Month, NaiveTime, Weekday};
use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum Date {
    MonthWeekDay {
        month: Month,
        week: u32,
        day: Weekday,
    },
    MonthDay {
        month: Month,
        day: u32,
    },
}

#[derive(Clone, Deserialize, Debug)]
pub struct Hours {
    weekday: Weekday,
    start: NaiveTime,
    stop: NaiveTime,
}

impl Hours {
    pub fn weekday(&self) -> Weekday {
        self.weekday
    }

    pub fn start(&self) -> NaiveTime {
        self.start
    }

    pub fn stop(&self) -> NaiveTime {
        self.stop
    }
}

impl Default for Hours {
    fn default() -> Self {
        Self {
            weekday: Weekday::Mon,
            start: NaiveTime::from_hms(0, 0, 0),
            stop: NaiveTime::from_hms(23, 59, 59),
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


use crate::application::config::{DateTimeConfig, Hours, Date::{MonthWeekDay, MonthDay}};

use std::time::Duration;
use std::collections::HashSet;

use num_traits::FromPrimitive;

use chrono::{Date, Datelike, Local, NaiveDate, Timelike, Weekday};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DateError {
    #[error("{0} of value {0} is out of range")]
    OutOfRangeError(String, u8),

    #[error("{0} has its start time > stop time")]
    FlipFlopTimeError(Weekday),
    
    #[error("{0} was stated multiple times in config for start times")]
    RepeatedWeekdayError(Weekday),
    
    #[error("Day of the week is over 7 or is 0")]
    DayOfTheWeekError,

    #[error("Tried to find next start time while running")]
    RunningError,

    #[error("Tried to find next stop time while not running")]
    NotRunningError,
}

type DateResult<T> = Result<T, DateError>;

fn num_to_weekday(d: u8) -> DateResult<Weekday> {
    if d > 0 {
        let weekday = Weekday::from_u64((d-1) as u64);
        if let Some(w) = weekday { return Ok(w); }
    }
    Err(DateError::DayOfTheWeekError)
}

fn weekdays_hours(weekday: Weekday, config: &DateTimeConfig) -> Hours {
    for hours in config.start_hours() {
        if weekday == hours.0 {
            return *hours;
        }
    }
    (weekday, 0, 24)
}

// I am not going to check in depth
// Unless there is a library I can use to easily do it
pub fn sanity_check(config: &DateTimeConfig) -> DateResult<()> {
    for date in config.dates() {
        match *date {
            MonthWeekDay { month, week, day } => {
                if month == 0 || month > 12 {
                    return Err(DateError::OutOfRangeError("Month".to_owned(), month));
                } else if week == 0 || week > 5 {
                    return Err(DateError::OutOfRangeError("Week".to_owned(), week));
                } else if day == 0 || day > 7 {
                    return Err(DateError::OutOfRangeError("Day".to_owned(), day));
                }
            }
            MonthDay { month, day } => {
                if month == 0 || month > 12 {
                    return Err(DateError::OutOfRangeError("Month".to_owned(), month));
                } else if day == 0 || day > 31 {
                    return Err(DateError::OutOfRangeError("Day".to_owned(), day));
                }
            }
        }
    }

    let mut weekday_set = HashSet::new();

    for hours in config.start_hours() {
        if !weekday_set.contains(&hours.0) {
            weekday_set.insert(hours.0.clone());
        } else {
            return Err(DateError::RepeatedWeekdayError(hours.0.clone()));
        }
        if hours.1 >= hours.2 {
            return Err(DateError::FlipFlopTimeError(hours.0.clone()));
        }
    }

    Ok(())
}

pub fn should_run(date: &Date<Local>, config: &DateTimeConfig) -> DateResult<bool> {
    let from_ymwd = NaiveDate::from_weekday_of_month;
    let from_ymd = NaiveDate::from_ymd;

    for d in config.dates() {
        match *d {
            MonthWeekDay { month, week, day } => {
                if  date.naive_local() == 
                    from_ymwd(date.year(), month as u32, num_to_weekday(week)?, day)
                {
                    return Ok(false);
                }
            }
            MonthDay { month, day } => {
                if  date.naive_local() == from_ymd(date.year(), month as u32, day as u32) {
                    return Ok(false);
                }
            }
        }
    } 
    Ok(true)
}

pub fn next_start_time(config: &DateTimeConfig) -> DateResult<Duration> {
    let now = Local::now(); 
    let mut weekday = now.weekday();
    let hours = weekdays_hours(weekday, config);
    
    let run_today = should_run(&now.date(), config)?; 

    if run_today && now.hour() < hours.1 as u32 {
        let diff = (hours.1 as u32 - now.hour())*3600 - now.minute()*60 - now.second();
        return Ok(Duration::from_secs(diff as u64));
    } else if !run_today || now.hour() >= hours.2 as u32 {
        weekday = weekday.succ();
        let mut next_day = now.date().succ();
        let mut time = (24 - now.hour())*3600 - now.minute()*60 - now.second();

        while !should_run(&next_day, config)? {
            time += 24 * 3600;
            weekday = weekday.succ();
            next_day = next_day.succ();
        }
        
        let hours = weekdays_hours(weekday, config);
        time += hours.1 as u32 * 3600;

        return Ok(Duration::from_secs(time as u64));
    }
    
    Err(DateError::RunningError)
}

pub fn next_stop_time(config: &DateTimeConfig) -> DateResult<Duration> {
    let now = Local::now(); 
    let mut weekday = now.weekday();
    let mut hours = weekdays_hours(weekday, config);
    
    let run_today = should_run(&now.date(), config)?; 

    if run_today && now.hour() >= hours.1 as u32 && now.hour() < hours.2 as u32 {
        let mut time = (hours.2 as u32 - now.hour())*3600 - now.minute()*60 - now.second();
        if hours.2 == 24 { 
            weekday = weekday.succ();
            let mut next_day = now.date().succ();
            hours = weekdays_hours(weekday, config);

            while should_run(&next_day, config)? && hours.1 == 0 && hours.2 == 24 {
                time += 24 * 3600; 
                weekday = weekday.succ();
                next_day = next_day.succ();
                hours = weekdays_hours(weekday, config);
            }
            
            time += hours.1 as u32 * 3600;
        }
        return Ok(Duration::from_secs(time as u64));
    }     
    
    Err(DateError::NotRunningError)
}


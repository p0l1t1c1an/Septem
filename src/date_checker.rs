use crate::config::date_config::{
    Date::{MonthDay, MonthWeekDay},
    DateTimeConfig, Hours,
};

use std::collections::HashSet;
use std::time::Duration;
use tokio::time::{sleep_until, Instant};

use chrono::{Date, Datelike, Local, NaiveDate, Timelike, Weekday};
use thiserror::Error;

use StartStopTimes::{EndOfDay, EndOfMonitoring, StartOfMonitoring};

#[derive(Error, Debug)]
pub enum DateError {
    #[error("{0} doesn't have {1} number of days")]
    DayOfMonthError(&'static str, u32),

    #[error("{0} doesn't have {2} number of {1}s")]
    WeekdayOfMonthError(&'static str, Weekday, u32),

    #[error("{0} has its start time > stop time")]
    FlipFlopTimeError(Weekday),

    #[error("{0} was stated multiple times in config for start times")]
    RepeatedWeekdayError(Weekday),

    #[error("A weekday has one or more of its hours set higher than 24")]
    HoursTooHighError,
}

type DateResult<T> = Result<T, DateError>;

pub enum StartStopTimes {
    EndOfDay(Duration, bool),
    StartOfMonitoring(Duration),
    EndOfMonitoring(Duration),
}

fn weekdays_hours(weekday: Weekday, config: &DateTimeConfig) -> (u32, u32) {
    for hours in config.start_hours() {
        if weekday == hours.weekday() {
            return (hours.start(), hours.stop());
        }
    }

    let default = Hours::default();
    (default.start(), default.stop())
}

pub fn sanity_check(config: &DateTimeConfig) -> DateResult<()> {
    let opt_from_ymwd = NaiveDate::from_weekday_of_month_opt;
    let opt_from_ymd = NaiveDate::from_ymd_opt;
    let today = Local::today();

    for date in config.dates() {
        match *date {
            MonthWeekDay { month, week, day } => {
                let m = month.number_from_month();
                if let None = opt_from_ymwd(today.year(), m, day, week as u8) {
                    return Err(DateError::WeekdayOfMonthError(month.name(), day, week));
                }
            }
            MonthDay { month, day } => {
                let m = month.number_from_month();
                if let None = opt_from_ymd(today.year(), m, day) {
                    return Err(DateError::DayOfMonthError(month.name(), day));
                }
            }
        }
    }

    let mut weekday_set = HashSet::new();

    for hours in config.start_hours() {
        if !weekday_set.contains(&hours.weekday()) {
            weekday_set.insert(hours.weekday().clone());
        } else {
            return Err(DateError::RepeatedWeekdayError(hours.weekday()));
        }

        if hours.start() >= hours.stop() {
            return Err(DateError::FlipFlopTimeError(hours.weekday()));
        } else if hours.start() > 24 || hours.stop() > 24 {
            return Err(DateError::HoursTooHighError);
        }
    }

    Ok(())
}

fn should_run(date: &Date<Local>, config: &DateTimeConfig) -> bool {
    let from_ymwd = NaiveDate::from_weekday_of_month;
    let from_ymd = NaiveDate::from_ymd;

    for d in config.dates() {
        match *d {
            MonthWeekDay { month, week, day } => {
                if date.naive_local()
                    == from_ymwd(date.year(), month.number_from_month(), day, week as u8)
                {
                    return false;
                }
            }
            MonthDay { month, day } => {
                if date.naive_local() == from_ymd(date.year(), month.number_from_month(), day) {
                    return false;
                }
            }
        }
    }
    true
}

fn next_time(config: &DateTimeConfig) -> StartStopTimes {
    let now = Local::now();
    let weekday = now.weekday();
    let (start, stop) = weekdays_hours(weekday, config);

    let run_today = should_run(&now.date(), config);

    if run_today {
        if now.hour() < start {
            let time = (start - now.hour()) * 3600 - now.minute() * 60 - now.second();
            return StartStopTimes::StartOfMonitoring(Duration::from_secs(time as u64));
        } else if now.hour() >= start && now.hour() < stop {
            let time = (stop - now.hour()) * 3600 - now.minute() * 60 - now.second();
            if stop == 24 {
                return StartStopTimes::EndOfDay(Duration::from_secs(time as u64), true);
            } else {
                return StartStopTimes::EndOfMonitoring(Duration::from_secs(time as u64));
            }
        }
    }

    let time = (24 - now.hour()) * 3600 - now.minute() * 60 - now.second();
    StartStopTimes::EndOfDay(Duration::from_secs(time as u64), false)
}

pub async fn wait_next(config: DateTimeConfig) -> bool {
    let next = next_time(&config);
    match next {
        EndOfDay(d, _) => {
            sleep_until(Instant::now() + d).await;
            match next_time(&config) {
                EndOfDay(_, is_on) => is_on,
                EndOfMonitoring(_) => true,
                StartOfMonitoring(_) => false,
            }
        }
        EndOfMonitoring(d) => {
            sleep_until(Instant::now() + d).await;
            false
        }
        StartOfMonitoring(d) => {
            sleep_until(Instant::now() + d).await;
            true
        }
    }
}

pub async fn wait_start(config: DateTimeConfig) {
    let mut next = next_time(&config);
    loop {
        match next {
            EndOfDay(d, is_on) => {
                if !is_on {
                    sleep_until(Instant::now() + d).await;
                } else {
                    break;
                }
            }
            EndOfMonitoring(_) => {
                break;
            }
            StartOfMonitoring(d) => {
                sleep_until(Instant::now() + d).await;
            }
        }
        next = next_time(&config);
    }
}

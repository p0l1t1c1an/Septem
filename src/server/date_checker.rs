use crate::config::date_config::{
    Date::{MonthDay, MonthWeekDay},
    DateTimeConfig, Hours,
};
use crate::server::client::{Client, ClientResult, Running, Timeout};

use std::collections::HashSet;
use std::time::Duration;

use chrono::{Date, Datelike, Local, NaiveDate, NaiveTime, Timelike, Weekday};

use async_trait::async_trait;
use thiserror::Error;

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
}

type DateResult<T> = Result<T, DateError>;

#[derive(Debug)]
enum StartStopTimes {
    EndOfDay(Duration, bool),
    StartOfAlerts(Duration),
    EndOfAlerts(Duration),
}

pub struct DateChecker {
    config: DateTimeConfig,
    running: Running,
    alerts_on: Running,
    timeout: Timeout,
}

impl DateChecker {
    pub fn new(
        config: DateTimeConfig,
        running: Running,
        alerts_on: Running,
        timeout: Timeout,
    ) -> DateResult<DateChecker> {
        Self::sanity_check(&config)?;
        Ok(DateChecker {
            config,
            running,
            alerts_on,
            timeout,
        })
    }

    fn sanity_check(config: &DateTimeConfig) -> DateResult<()> {
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
            }
        }

        Ok(())
    }

    fn weekdays_hours(&self, weekday: Weekday) -> (NaiveTime, NaiveTime) {
        for hours in self.config.start_hours() {
            if weekday == hours.weekday() {
                return (hours.start(), hours.stop());
            }
        }

        let default = Hours::default();
        (default.start(), default.stop())
    }

    fn should_run(&self, date: &Date<Local>) -> bool {
        let from_ymwd = NaiveDate::from_weekday_of_month;
        let from_ymd = NaiveDate::from_ymd;

        for d in self.config.dates() {
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

    fn next_time(&self) -> StartStopTimes {
        let now = Local::now();
        let weekday = now.weekday();
        let (start, stop) = self.weekdays_hours(weekday);

        let run_today = self.should_run(&now.date());

        if run_today {
            if now.time() < start {
                let time = start - now.time();
                return StartStopTimes::StartOfAlerts(Duration::from_secs(
                    time.num_seconds() as u64
                ));
            } else if now.time() >= start && now.time() < stop {
                let time = stop - now.time();
                if stop == NaiveTime::from_hms(23, 59, 59) {
                    return StartStopTimes::EndOfDay(
                        Duration::from_secs(time.num_seconds() as u64),
                        true,
                    );
                } else {
                    return StartStopTimes::EndOfAlerts(Duration::from_secs(
                        time.num_seconds() as u64
                    ));
                }
            }
        }

        let time = (24 - now.hour()) * 3600 - now.minute() * 60 - now.second();
        StartStopTimes::EndOfDay(Duration::from_secs(time as u64), false)
    }
}

#[async_trait]
impl Client for DateChecker {
    async fn start(mut self) -> ClientResult<()> {
        use StartStopTimes::*;
        while self.running.load() {
            match self.next_time() {
                StartOfAlerts(d) => {
                    self.alerts_on.store(false);
                    self.timeout.wait_timeout(d).await?;
                }
                EndOfAlerts(d) => {
                    self.alerts_on.store(true);
                    self.timeout.wait_timeout(d).await?;
                }
                EndOfDay(d, is_running) => {
                    self.alerts_on.store(is_running);
                    self.timeout.wait_timeout(d).await?;
                }
            }
        }
        Ok(())
    }
}

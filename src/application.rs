mod alert;
mod client;
mod event_handler;
mod recorder;
mod signal_handler;

use crate::config::{Config, ConfigError, date_config::DateTimeConfig};
use crate::date_checker::{self, DateError};

use alert::{AlertError, Alerter};
use client::{Client, ClientError, Condition, Pid, Productive, Running};
use event_handler::{EventError, EventHandler};
use recorder::{Recorder, RecorderError};
use signal_handler::{SignalError, SignalHandler};

use futures::future::{try_join_all, select, Either};
use tokio::spawn;
use tokio::task::{JoinError, JoinHandle};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    JoinAllError(#[from] JoinError),

    #[error("{0}")]
    RunningClientError(#[from] ClientError),

    #[error("{0}")]
    StartUpAlertError(#[from] AlertError),

    #[error("{0}")]
    StartUpConfigError(#[from] ConfigError),

    #[error("{0}")]
    StartUpDateError(#[from] DateError),

    #[error("{0}")]
    StartUpRecorderError(#[from] RecorderError),

    #[error("{0}")]
    StartUpEventError(#[from] EventError),

    #[error("{0}")]
    StartUpSignalError(#[from] SignalError),
}

type AppResult<T> = Result<T, AppError>;

type ClientThread = JoinHandle<Result<(), ClientError>>;

async fn restart(running: &Running) -> AppResult<(DateTimeConfig, Vec<ClientThread>)> {
    let (share, rec_conf, date_conf, alert_conf) = Config::new(None)?.break_up()?;
    date_checker::sanity_check(&date_conf)?;

    let pid = Pid::new();
    let cond = Condition::new();
    let prod = Productive::new(false);

    let event = EventHandler::new(pid.clone(), running.clone(), cond.clone())?;
    let signal = SignalHandler::new(running.clone(), cond)?;

    let recorder = Recorder::new(share, rec_conf, pid, running.clone(), prod.clone())?;
    let alert = Alerter::new(alert_conf, running.clone(), prod)?;

    let clients = vec![
        spawn(event.start()),
        spawn(signal.start()),
        spawn(recorder.start()),
        spawn(alert.start()),
    ];
    
    Ok((date_conf, clients))
}

pub async fn start() -> AppResult<()> {
    let running = Running::new(false);
    let (mut date_conf, mut clients) = restart(&running).await?;

    let mut joined = spawn(try_join_all(clients));
    let mut next = spawn(date_checker::wait_next(date_conf.clone()));

    loop {
        match select(joined, next).await {
            Either::Left((j, _)) => { 
                for error in j??.into_iter() {
                    error?;
                }
                break;
            }
            Either::Right((n, j)) => {
                if running.load() { break; }

                if n? {
                    joined = j;
                    next = spawn(date_checker::wait_next(date_conf.clone()));
                } else { 

                    // TODO: Stop Clients
                    
                    for error in j.await??.into_iter() {
                        error?;
                    }
                    while !date_checker::wait_next(date_conf.clone()).await { }
                    
                    let reset = restart(&running).await?;
                    date_conf = reset.0;
                    clients = reset.1;
                    
                    joined = spawn(try_join_all(clients));
                    next = spawn(date_checker::wait_next(date_conf.clone())); 
                }
            }
        }
    }

    println!("App End");
    Ok(())
}

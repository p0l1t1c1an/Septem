mod alert;
mod client;
mod config;
mod date_checker;
mod event_handler;
mod process;
mod recorder;
mod signal_handler;

use alert::{AlertError, Alerter};
use client::{Client, ClientError, Condition, Pid, Productive, Shutdown, Running};
use config::{Config, ConfigError};
use date_checker::DateError;
use event_handler::{EventError, EventHandler};
use recorder::{Recorder, RecorderError};
use signal_handler::{SignalError, SignalHandler};

use futures::future::{Either, select, try_join_all};
use tokio::spawn;
use tokio::task::JoinError;

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

pub async fn start() -> AppResult<()> {
    let config = Config::new(None)?;
    date_checker::sanity_check(&config.date_config())?;
    let pid = Pid::new();
    let shut = Shutdown::new(false);
    let run = Running::new(true);
    let cond = Condition::new();
    let prod = Productive::new(false);

    let event = EventHandler::new(pid.clone(), shut.clone(), cond.clone())?;
    let signal = SignalHandler::new(shut.clone(), run.clone(), cond)?;

    let recorder = Recorder::new(
        config.share()?,
        config.recorder_config(),
        pid,
        shut.clone(),
        prod,
    )?;
    let alert = Alerter::new(config.alert_config(), shut, prod)?;
    
    drop(config);

    let singal_handle = spawn(signal.start());

    let clients = vec![
        spawn(event.start()),
        spawn(recorder.start()),
        spawn(alert.start()),
    ];

    // TODO: Spawn thread that is sleeping 
    // and using date checker to wait and then send a sighup to 
    // flip shutdown. Will need to use a sigterm ... to close loop 
    // that is a select of the try_join_all below and new thread


    let joined = try_join_all(clients);
    
    while run.load() {
        match select(joined, singal_handle).await {
            Either::Left((left, right)) => {
                
            }
            Either::Right((right, left)) => {

            }
        }
    }


    for error in errors.into_iter() {
        error?; // Is is Ok or Err
    }
    

    println!("App End");
    Ok(())
}

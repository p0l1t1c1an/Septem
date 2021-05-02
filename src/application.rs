mod alert;
mod client;
mod config;
mod date_checker;
mod event_handler;
mod process;
mod recorder;
mod signal_handler;

use alert::{AlertError, Alerter};
use client::{Client, ClientError};
use config::{Config, ConfigError};
use date_checker::DateError;
use event_handler::{EventError, EventHandler};
use recorder::{Recorder, RecorderError};
use signal_handler::{SignalError, SignalHandler};

use futures::future::try_join_all;
use tokio::spawn;
use tokio::sync::mpsc::channel;
use tokio::task::JoinError;

use std::sync::{atomic::AtomicBool, Arc, Condvar, Mutex};

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
    let pid = Arc::new((Mutex::new(None), Condvar::new()));
    let shut = Arc::new(AtomicBool::new(false));
    let cond = Arc::new((Mutex::new(()), Condvar::new()));
    let (tx, rx) = channel(1);

    let event = EventHandler::new(Arc::clone(&pid), Arc::clone(&shut), Arc::clone(&cond))?;
    let signal = SignalHandler::new(Arc::clone(&shut), cond)?;

    let recorder = Recorder::new(
        config.share()?,
        config.recorder_config(),
        pid,
        shut,
        tx,
    )?;
    let alert = Alerter::new(config.alert_config(), rx)?;
    
    drop(config);

    let join_clients = vec![
        spawn(event.start()),
        spawn(signal.start()),
        spawn(recorder.start()),
        spawn(alert.start()),
    ];

    let errors = try_join_all(join_clients).await?;
    for error in errors.into_iter() {
        error?; // Is is Ok or Err
    }
    println!("App End");
    Ok(())
}

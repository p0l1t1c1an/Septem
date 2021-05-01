mod alert;
mod client;
mod config;
mod event_handler;
mod process;
mod recorder;
mod signal_handler;

use alert::{AlertError, Alerter};
use client::{Client, ClientError};
use config::{Config, ConfigError};
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
    StartUpRecorderError(#[from] RecorderError),

    #[error("{0}")]
    StartUpEventError(#[from] EventError),

    #[error("{0}")]
    StartUpSignalError(#[from] SignalError),
}

type AppResult<T> = Result<T, AppError>;

pub async fn start() -> AppResult<()> {
    let c = Config::new(None)?;
    let pid = Arc::new((Mutex::new(None), Condvar::new()));
    let shut = Arc::new(AtomicBool::new(false));
    let cond = Arc::new((Mutex::new(()), Condvar::new()));
    let (tx, rx) = channel(1);

    let e = EventHandler::new(Arc::clone(&pid), Arc::clone(&shut), Arc::clone(&cond))?;
    let s = SignalHandler::new(Arc::clone(&shut), Arc::clone(&cond))?;

    let r = Recorder::new(
        c.shared_dir()?,
        c.recorder_config(),
        Arc::clone(&pid),
        Arc::clone(&shut),
        tx,
    )?;
    let a = Alerter::new(c.alert_config(), rx)?;

    let join_clients = vec![
        spawn(e.start()),
        spawn(s.start()),
        spawn(r.start()),
        spawn(a.start()),
    ];

    let errors = try_join_all(join_clients).await?;
    for error in errors.into_iter() {
        error?; // Is is Ok or Err
    }
    println!("App End");
    Ok(())
}

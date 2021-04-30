mod config;
mod event_handler;
mod process;
mod recorder;
pub mod server;
mod signal_handler;

use config::{Config, ConfigError};
use event_handler::{EventError, EventHandler};
use recorder::{Recorder, RecorderError};
use server::{Client, ClientError, Server};
use signal_handler::{SignalError, SignalHandler};

use futures::future::try_join_all;
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
    StartUpConfigError(#[from] ConfigError),

    #[error("{0}")]
    StartUpRecorderError(#[from] RecorderError),

    #[error("{0}")]
    StartUpEventError(#[from] EventError),

    #[error("{0}")]
    StartUpSignalError(#[from] SignalError),
}

type AppResult<T> = Result<T, AppError>;

pub async fn init() -> AppResult<Server> {
    let c = Config::new(None)?;
    println!("{:#?}", c);

    let r = Recorder::new(c.shared_dir()?, c.recorder_config())?;
    let e = EventHandler::new()?;
    let s = SignalHandler::new()?;

    let pid = Arc::new((Mutex::new(None), Condvar::new()));
    let shutdown = Arc::new((AtomicBool::new(false), Mutex::new(()), Condvar::new()));

    Ok(Server::new(vec![
        Client::EventClient(Arc::clone(&pid), Arc::clone(&shutdown), e),
        Client::RecorderClient(pid, Arc::clone(&shutdown), r),
        Client::SignalClient(shutdown, s),
    ]))
}

pub async fn start(server: Server) -> Result<(), AppError> {
    let join_clients = server.start_clients().await;
    let errors = try_join_all(join_clients).await?;
    for e in errors.into_iter() {
        e?;
    }
    println!("App End");
    Ok(())
}

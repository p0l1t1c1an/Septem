mod config;
pub mod event_handler;
mod process;
mod recorder;
pub mod server;

use event_handler::{EventHandler, EventError};
use recorder::{Recorder, RecorderError};
use server::{ClientError, Server};

use futures::future::try_join_all;
use tokio::task::JoinError;

use std::sync::{Arc, Mutex};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {  
    #[error("{0}")]
    JoinAllError(#[from] JoinError),

    #[error("{0}")]
    RunningClientError(#[from] ClientError),

    #[error("{0}")]
    StartUpRecorderError(#[from] RecorderError),

    #[error("{0}")]
    StartUpEventError(#[from] EventError),
}

type AppResult<T> = Result<T, AppError>; 


pub async fn init() -> AppResult<Server> {
    let r = Recorder::new("/usr/home/p0l1t1c1an/.local/share/Septem".to_owned()).await?; 
    let e = EventHandler::new(10)?;
    let p = Arc::new(Mutex::new(0));

    let mut v = Vec::new();
    v.push(server::Client::EventClient(p, e));

    Ok(Server::new(v))
}

pub async fn start(server : Server) -> Result<(), AppError> { 
    let join_clients = server.start_clients().await;
    let errors = try_join_all(join_clients).await?;
    for e in errors.into_iter() {
        e?;
    }
    Ok(())
}

#![allow(dead_code)]

use crate::application::{
    alert::AlertError,
    event_handler::EventError,
    recorder::RecorderError,
    signal_handler::SignalError,
};

use std::sync::{atomic::AtomicBool, Arc, Condvar, Mutex};

use tokio::task::JoinError;
use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("{0}")]
    JoinThreadError(#[from] JoinError),
    
    #[error("{0}")]
    AlertClientError(#[from] AlertError),

    #[error("{0}")]
    EventClientError(#[from] EventError),

    #[error("{0}")]
    RecorderClientError(#[from] RecorderError),
    
    #[error("{0}")]
    SignalClientError(#[from] SignalError),
}

pub type ClientResult = Result<(), ClientError>;
pub type Pid = Arc<(Mutex<Option<u32>>, Condvar)>;
pub type Shutdown = Arc<AtomicBool>;
pub type Condition = Arc<(Mutex<()>, Condvar)>;


#[async_trait]
pub trait Client {
    async fn start(self) -> ClientResult;
}


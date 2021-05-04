#![allow(dead_code)]

use crate::application::{
    alert::AlertError, event_handler::EventError, recorder::RecorderError,
    signal_handler::SignalError,
};

use std::sync::{atomic::{AtomicBool, Ordering}, Arc, Condvar, Mutex};
use tokio::sync::Notify;

use async_trait::async_trait;
use thiserror::Error;
use tokio::task::JoinError;

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

    #[error("The {0} mutex failed to lock")]
    PosionedMutexError(&'static str),

    #[error("The {0} condvar failed to load")]
    PosionedCondvarError(&'static str),

}

pub type ClientResult<T> = Result<T, ClientError>;

// Todo: Possibly reimplement Pid using RwLock and Notify over Mutex and Condvar


#[derive(Clone, Debug)]
pub struct Pid {
    val: Arc<(Mutex<Option<u32>>, Condvar)>,
}

impl Pid {
    pub fn new() -> Self {
        Pid {
            val: Arc::new((Mutex::new(None), Condvar::new()))
        }
    }

    pub fn set_pid(&self, val: Option<u32>) -> ClientResult<()> {
        match self.val.0.lock() {
            Ok(mut v) => {
                *v = val;
                Ok(())
            }
            Err(_) => {
                Err(ClientError::PosionedMutexError("Pid"))
            }
        }
    } 

    pub fn notify_one(&self) {
        self.val.1.notify_one()
    }

    pub fn get_pid(&self) -> ClientResult<Option<u32>> {
        match self.val.0.lock() {
            Ok(guard) => match self.val.1.wait(guard) {
                Ok(v) => {
                    Ok(v)
                }
                Err(_) => {
                    return Err(ClientError::PosionedCondvarError("Pid"));
                }
            }
            Err(_) => {
                return Err(ClientError::PosionedMutexError("Pid"));
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Shutdown {
    val: Arc<AtomicBool>
}

// Each type is used for different purpose
// Lets me know what to use the variable for
pub type Running = Shutdown;
pub type Productive = Shutdown;

impl Shutdown {
    pub fn new(val: bool) -> Self {
        Self {
            val: Arc::new(AtomicBool::new(val))
        }
    }
    
    pub fn load(&self) -> bool {
        self.val.load(Ordering::SeqCst)
    }

    pub fn store(&self, val: bool) {
        self.val.store(val, Ordering::SeqCst);
    }
}

#[derive(Clone, Debug)]
pub struct Condition {
    val: Arc<Notify>
}

impl Condition {
    pub fn new() -> Self {
        Self {
            val: Arc::new(Notify::new())
        }
    }

    pub fn notify_one(&self) {
        self.val.notify_one();
    }

    pub async fn wait(&self) {
        self.val.notified().await;
    }
}

#[async_trait]
pub trait Client {
    async fn start(self) -> ClientResult<()>;
}

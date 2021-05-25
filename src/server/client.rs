#![allow(dead_code)]

use crate::server::{
    alert::AlertError, event_handler::EventError, recorder::RecorderError,
    signal_handler::SignalError,
};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{Notify, RwLock};
use tokio::time::{sleep_until, Duration, Instant};

use futures::future::{select, Either};
use futures::pin_mut;

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
}

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Debug)]
pub struct Pid (pub Sender<Option<u32>>, pub Receiver<Option<u32>>);

unsafe impl Send for Pid {}
unsafe impl Sync for Pid {}

impl Pid {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel(1);
        Pid (tx, rx)
    }
}

pub type PidSender = Sender<Option<u32>>;
pub type PidRecv = Receiver<Option<u32>>;

#[derive(Clone, Debug)]
pub struct Running {
    val: Arc<AtomicBool>,
}

unsafe impl Send for Running {}
unsafe impl Sync for Running {}

// Each type is used for different purpose
// Lets me know what to use the variable for
pub type Productive = Running;

impl Running {
    pub fn new(val: bool) -> Self {
        Self {
            val: Arc::new(AtomicBool::new(val)),
        }
    }

    pub fn load(&self) -> bool {
        self.val.load(Ordering::SeqCst)
    }

    pub fn store(&self, val: bool) {
        self.val.store(val, Ordering::SeqCst);
    }
}

pub enum WaitTimeout {
    Notified,
    TimedOut,
}

#[derive(Clone, Debug)]
pub struct Timeout {
    val: Arc<Notify>,
}

unsafe impl Send for Timeout {}
unsafe impl Sync for Timeout {}

impl Timeout {
    pub fn new() -> Self {
        Self {
            val: Arc::new(Notify::new()),
        }
    }

    pub fn notify_one(&self) {
        self.val.notify_one();
    }

    // Returns true if notify is notified
    // false if timeout
    // Could use an enum for which happened
    pub async fn wait_timeout(&self, time: Duration) -> WaitTimeout {
        let wait = self.val.notified();
        let sleep = sleep_until(Instant::now() + time);

        pin_mut!(wait);
        pin_mut!(sleep);

        match select(wait, sleep).await { 
            Either::Left(_) => WaitTimeout::Notified,
            Either::Right(_) => WaitTimeout::TimedOut,
        }
    }
}

#[async_trait]
pub trait Client {
    async fn start(self) -> ClientResult<()>;
}


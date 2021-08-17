#![allow(dead_code)]

use crate::server::{
    alert::AlertError, event_handler::EventError, recorder::RecorderError,
    signal_handler::SignalError,
};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::{mpsc, Notify};
use tokio::time::{sleep_until, Duration, Instant};

use futures::future::FutureExt;
use futures::select_biased;

use async_trait::async_trait;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Timeout was interrupted")]
    TimeoutError,

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

pub type PidSender = mpsc::Sender<Option<u32>>;
pub type PidRecv = mpsc::Receiver<Option<u32>>;

#[derive(Debug)]
pub struct Pid(pub PidSender, pub PidRecv);

unsafe impl Send for Pid {}
unsafe impl Sync for Pid {}

impl Pid {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(2);
        Pid(tx, rx)
    }
}

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

#[derive(Clone, Debug)]
pub struct Timeout {
    notify: Arc<Notify>,
}

unsafe impl Send for Timeout {}
unsafe impl Sync for Timeout {}

impl Timeout {
    pub fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn notify_all(&self) {
        self.notify.notify_waiters();
        self.notify.notify_one();
    }

    pub async fn wait(&self) {
        self.notify.notified().await;
    }

    pub async fn wait_timeout(&self, time: Duration) -> ClientResult<()> {
        select_biased! {
            _ = self.wait().fuse() => Err(ClientError::TimeoutError),
            _ = sleep_until(Instant::now() + time).fuse() => Ok(()),
        }
    }
}

#[async_trait]
pub trait Client {
    async fn start(self) -> ClientResult<()>;
}

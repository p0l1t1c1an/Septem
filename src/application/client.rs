#![allow(dead_code)]

use crate::application::{
    alert::AlertError, event_handler::EventError, recorder::RecorderError,
    signal_handler::SignalError,
};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
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

#[derive(Clone, Debug)]
pub struct Pid {
    val: Arc<(RwLock<Option<u32>>, Notify)>,
}

unsafe impl Send for Pid {}
unsafe impl Sync for Pid {}

impl Pid {
    pub fn new() -> Self {
        Pid {
            val: Arc::new((RwLock::new(None), Notify::new())),
        }
    }

    pub async fn set_pid(&self, val: Option<u32>) {
        let mut v = self.val.0.write().await;
        *v = val;
    }

    pub fn notify_one(&self) {
        self.val.1.notify_one()
    }

    pub async fn wait_pid(&self) -> Option<u32> {
        self.val.1.notified().await;
        *self.val.0.read().await
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

pub enum WaitTimeout {
    Notified,
    TimedOut,
}

#[derive(Clone, Debug)]
pub struct Condition {
    val: Arc<Notify>,
}

unsafe impl Send for Condition {}
unsafe impl Sync for Condition {}

pub type Timeout = Condition;

impl Condition {
    pub fn new() -> Self {
        Self {
            val: Arc::new(Notify::new()),
        }
    }

    pub fn notify_one(&self) {
        self.val.notify_one();
    }

    pub async fn wait(&self) {
        self.val.notified().await;
    }

    // Returns true if notify is notified
    // false if timeout
    // Could use an enum for which happened
    pub async fn wait_timeout(&self, time: Duration) -> WaitTimeout {
        let wait = self.wait();
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

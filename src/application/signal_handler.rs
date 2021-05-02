use crate::application::client::{Client, ClientResult, Condition, Shutdown};

use futures::stream::StreamExt;
use signal_hook::consts::signal::*;
use signal_hook_tokio::{Handle, Signals};

use std::io;
use std::sync::atomic::Ordering;

use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SignalError {
    #[error("{0}")]
    SignalCreationError(#[from] io::Error),

    #[error("An unregistered error was caught and cannot be handled")]
    UnknownSignalError,
}

type SignalResult<T> = Result<T, SignalError>;

pub struct SignalHandler {
    shutdown: Shutdown,
    cond: Condition,
    signals: Signals,
    handle: Handle,
}

impl SignalHandler {
    pub fn new(shutdown: Shutdown, cond: Condition) -> SignalResult<SignalHandler> {
        let signals = Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT])?;
        let handle = signals.handle();

        Ok(SignalHandler {
            shutdown,
            cond,
            signals,
            handle,
        })
    }
}

#[async_trait]
impl Client for SignalHandler {
    async fn start(self) -> ClientResult {
        let mut signals = self.signals.fuse();
        while let Some(sig) = signals.next().await {
            match sig {
                SIGHUP | SIGTERM | SIGINT | SIGQUIT => {
                    self.shutdown.store(true, Ordering::SeqCst);
                    let (_, c) = &*self.cond;
                    c.notify_one();
                    self.handle.close();
                    break;
                }
                _ => { return Err(SignalError::UnknownSignalError.into()); }
            }
        }

        Ok(())
    }
}

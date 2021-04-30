use crate::application::client::{Client, ClientResult, Condition, Shutdown};

use futures::stream::StreamExt;
use signal_hook::consts::signal::*;
use signal_hook_tokio::{Handle, Signals};

use std::sync::atomic::Ordering;
use std::io;

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
        let sig = Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT])?;
        let hand = sig.handle();

        Ok(SignalHandler {
            shutdown: shutdown,
            cond: cond,
            signals: sig,
            handle: hand,
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
                _ => Err(SignalError::UnknownSignalError)?,
            }
        }

        Ok(())
    }
}

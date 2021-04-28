use futures::stream::StreamExt;
use signal_hook::consts::signal::*;
use signal_hook_tokio::{Handle, Signals};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use std::sync::{Condvar, Mutex};

use std::io;
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
    signals: Signals,
    handle: Handle,
}

impl SignalHandler {
    pub fn new() -> SignalResult<SignalHandler> {
        let sig = Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT])?;
        let hand = sig.handle();

        Ok(SignalHandler {
            signals: sig,
            handle: hand,
        })
    }

    pub async fn start(self, shutdown: Arc<(AtomicBool, Mutex<()>, Condvar)>) -> SignalResult<()> {
        let mut signals = self.signals.fuse();
        while let Some(sig) = signals.next().await {
            match sig {
                SIGHUP | SIGTERM | SIGINT | SIGQUIT => {
                    shutdown.0.store(true, Ordering::Relaxed);
                    let (_, _, c) = &*shutdown;
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

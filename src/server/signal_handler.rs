use crate::server::client::{Client, ClientResult, Running, Timeout};

use futures::stream::StreamExt;
use signal_hook::consts::signal::*;
use signal_hook_tokio::{Handle, Signals};

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
    running: Running,
    event_time: Timeout,
    stop_start_time: Timeout,
    signals: Signals,
    handle: Handle,
}

impl SignalHandler {
    pub fn new(running: Running, e_time: Timeout, ss_time: Timeout) -> SignalResult<SignalHandler> {
        let signals = Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT])?;
        let handle = signals.handle();

        Ok(SignalHandler {
            running,
            event_time: e_time,
            stop_start_time: ss_time,
            signals,
            handle,
        })
    }

    pub fn handle(&self) -> Handle {
        self.handle.clone()
    }
}

#[async_trait]
impl Client for SignalHandler {
    async fn start(self) -> ClientResult<()> {
        let mut signals = self.signals.fuse();
        while let Some(sig) = signals.next().await {
            match sig {
                SIGHUP | SIGTERM | SIGINT | SIGQUIT => {
                    self.running.store(false);
                    self.event_time.notify_one();
                    self.stop_start_time.notify_one();
                    self.handle.close();
                    break;
                }
                _ => {
                    return Err(SignalError::UnknownSignalError.into());
                }
            }
        }
        println!("Signal End");
        Ok(())
    }
}
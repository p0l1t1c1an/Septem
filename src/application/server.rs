#![allow(dead_code)]

use crate::application::{
    event_handler::{EventError, EventHandler},
    recorder::{Recorder, RecorderError},
    signal_handler::{SignalError, SignalHandler},
};

use std::sync::{atomic::AtomicBool, Arc, Condvar, Mutex};

use thiserror::Error;
use tokio::task::JoinHandle;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("{0}")]
    RecorderClientError(#[from] RecorderError),

    #[error("{0}")]
    EventClientError(#[from] EventError),

    #[error("{0}")]
    SignalClientError(#[from] SignalError),
}

pub type ClientResult<T> = Result<T, ClientError>;

type Pid = Arc<(Mutex<Option<u32>>, Condvar)>;
type Shutdown = Arc<AtomicBool>;
type Signal = Arc<(Mutex<()>, Condvar)>;

type JoinClients<T> = Vec<JoinHandle<ClientResult<T>>>;

pub enum Client {
    RecorderClient(Pid, Shutdown, Recorder),
    EventClient(Pid, Shutdown, Signal, EventHandler),
    SignalClient(Shutdown, Signal, SignalHandler),
}

pub struct Server {
    clients: Vec<Client>,
}

impl Client {
    pub async fn start(self) -> ClientResult<()> {
        match self {
            Client::RecorderClient(pid, shut, client) => client.start(pid, shut).await?,
            Client::EventClient(pid, shut, sig, client) => client.start(pid, shut, sig).await?,
            Client::SignalClient(shut, sig, client) => client.start(shut, sig).await?,
        }
        Ok(())
    }
}

impl Server {
    pub fn new(v: Vec<Client>) -> Self {
        Self { clients: v }
    }

    pub async fn start_clients(self) -> JoinClients<()> {
        let mut handles = Vec::new();
        for client in self.clients.into_iter() {
            handles.push(tokio::spawn(client.start()));
        }
        handles
    }
}

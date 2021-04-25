#[allow(dead_code)]
#[allow(unused_variables)]
use crate::application::{
    event_handler::{EventError, EventHandler},
    process,
    recorder::{Recorder, RecorderError},
    signal_handler::{SignalHandler, SignalError},
};

use std::sync::{Arc, Mutex, Condvar, atomic::AtomicBool};
use std::time::SystemTime;

use tokio::task::JoinHandle;

use thiserror::Error;

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

type PidMutex = Arc<(Mutex<u32>, Condvar)>;
type ShutdownMutex = Arc<AtomicBool>;
type ProcessMutex = Arc<Mutex<process::Process>>;
type TimeMutex = Arc<Mutex<SystemTime>>;

type JoinClients<T> = Vec<JoinHandle<ClientResult<T>>>;

pub enum Client {
    RecorderClient(PidMutex, ShutdownMutex, Recorder),
    EventClient(PidMutex, ShutdownMutex, EventHandler),
    SignalClient(ShutdownMutex, SignalHandler),
}

pub struct Server {
    clients: Vec<Client>,
}

impl Client {
    pub async fn start(self) -> ClientResult<()> {
        match self {
            Client::RecorderClient(p, s, r) => r.start(p, s).await?,
            Client::EventClient(p, s, e) => e.start(p, s).await?,
            Client::SignalClient(s_m, s_h) => s_h.start(s_m).await?,
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

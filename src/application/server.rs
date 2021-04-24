#[allow(dead_code)]
#[allow(unused_variables)]
use crate::application::{
    event_handler::{EventError, EventHandler},
    process,
    recorder::{Recorder, RecorderError},
};

use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use tokio::task::JoinHandle;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("{0}")]
    RecorderClientError(#[from] RecorderError),

    #[error("{0}")]
    EventClientError(#[from] EventError),
}

pub type ClientResult<T> = Result<T, ClientError>;

type PidMutex = Arc<Mutex<u32>>;
type ProcessMutex = Arc<Mutex<process::Process>>;
type TimeMutex = Arc<Mutex<SystemTime>>;

type JoinClients<T> = Vec<JoinHandle<ClientResult<T>>>;

pub enum Client {
    RecorderClient(PidMutex, Recorder),
    EventClient(PidMutex, EventHandler),
}

pub struct Server {
    clients: Vec<Client>,
}

impl Client {
    pub async fn start(self) -> ClientResult<()> {
        match self {
            Client::RecorderClient(p, r) => r.start(p).await?,
            Client::EventClient(p, e) => e.start(p).await?,
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

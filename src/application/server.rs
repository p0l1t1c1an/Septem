#[allow(dead_code)]
#[allow(unused_variables)]
use crate::application::{event_handler::EventError, process, recorder::RecorderError};

use async_trait::async_trait;
use std::time::SystemTime;
use tokio::sync::{mpsc, watch};

use thiserror::Error;

/*
 * TODO:
 *
 * Then, it should have one for each threads
 * own errors.
 *
 */

#[derive(Error, Debug)]
pub enum ClientServerError {
    #[error("The client {0} was requested data it doesn't have")]
    IncorrectDataRequest(u8, Data),

    #[error("The request to client {0} timed out")]
    TimeoutError(u8),

    #[error("The request to the server timed out")]
    ServerTimeoutError,

    #[error("{0}")]
    RecorderClientError(#[from] RecorderError),

    #[error("{0}")]
    EventClientError(#[from] EventError),
}

pub type ClientServerResult<T> = Result<T, ClientServerError>;

// There are more data things
// But I just can't think of them
#[derive(Debug)]
pub enum Data {
    GetCurrProc,
    ReturnCurrProc(process::Process),

    GetCurrProcStart,
    ReturnCurrProcStart(SystemTime),

    Interupt(u32),
    Shutdown,
}

pub enum Message {
    GetFromServer { from: u8 }, // Client to Server
    GetFromClient { to: u8 },   // Server to Client

    ReturnToServer { data: Data },
    ReturnToClient { to: u8, data: Data },

    SendToClient { to: u8, data: Data },

    Shutdown { data: Data }, // Tell clients to shutdown
}

/*
 * TODO:
 *
 * It should run tokio spawn on each start for a client.
 * Then, use join! macro to combine handle and return.
 *
 */

#[async_trait]
pub trait Client {
    fn for_me(&self, to: u8) -> bool;

    async fn handle_message(&self, message: Message) -> ClientServerResult<()>;

    async fn start(
        self,
        id: u8,
        sender: mpsc::Sender<Message>,
        receiver: watch::Receiver<Message>,
    ) -> ClientServerResult<()>;
}

#[async_trait]
pub trait Server {
    async fn handle_message(&self, message: Message) -> ClientServerResult<()>;

    async fn start_clients(&self, clients: Vec<Box<dyn Client>>) -> ClientServerResult<()>;

    async fn start(
        &self,
        sender: watch::Sender<Message>,
        receiver: mpsc::Receiver<Message>,
    ) -> ClientServerResult<()>;
}

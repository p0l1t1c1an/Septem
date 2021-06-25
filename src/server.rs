mod alert;
mod client;
mod date_checker;
mod event_handler;
mod recorder;
mod signal_handler;

use crate::config::{Config, ConfigError};

use alert::{AlertError, Alerter};
use client::{Client, ClientError, Pid, Productive, Running, Timeout};
use date_checker::{DateChecker, DateError};
use event_handler::{EventError, EventHandler};
use recorder::{Recorder, RecorderError};
use signal_handler::{SignalError, SignalHandler};

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{
    future::{try_join_all, TryJoinAll},
    FutureExt,
};
use signal_hook_tokio::Handle;
use tokio::spawn;
use tokio::task::{JoinError, JoinHandle};

use thiserror::Error;

use self::client::ClientResult;

#[derive(Debug, Error)]
pub enum ServerError {
    // Error when joining threads
    #[error("{0}")]
    JoinAllError(#[from] JoinError),

    // Errors when clients are running
    #[error("{0}")]
    RunningClientError(#[from] ClientError),

    // Errors when creating clients
    #[error("{0}")]
    StartUpAlertError(#[from] AlertError),

    #[error("{0}")]
    StartUpConfigError(#[from] ConfigError),

    #[error("{0}")]
    StartUpDateError(#[from] DateError),

    #[error("{0}")]
    StartUpRecorderError(#[from] RecorderError),

    #[error("{0}")]
    StartUpEventError(#[from] EventError),

    #[error("{0}")]
    StartUpSignalError(#[from] SignalError),
}

pub type ServerResult<T> = Result<T, ServerError>;
type ClientThread = JoinHandle<ClientResult<()>>;

pub struct Server {
    config_file: Option<String>,
    config: Config,
    running: Running,
    timeout: Timeout,
    window: xcb::Window,
    sig_handle: Handle,
    clients: TryJoinAll<ClientThread>,
}

impl Server {
    pub fn new(config_file: Option<String>) -> ServerResult<Server> {
        let config = Config::new(config_file.clone())?;
        let share = config.share()?;
        let a_conf = config.alert_config();
        let d_conf = config.date_config();
        let r_conf = config.recorder_config();

        let running = Running::new(true);
        let timeout = Timeout::new();

        let pid = Pid::new();
        let prod = Productive::new(false);
        let alerts_on = Running::new(true);

        let event = EventHandler::new(pid.0.clone(), running.clone())?;
        let window = event.window();

        let signal = SignalHandler::new(running.clone())?;
        let recorder = Recorder::new(share, r_conf, pid.1, running.clone(), prod.clone())?;
        let date = DateChecker::new(d_conf, running.clone(), alerts_on.clone(), timeout.clone())?;
        let alert = Alerter::new(a_conf, running.clone(), alerts_on, prod)?;

        let sig_handle = signal.handle();
        let clients = try_join_all(vec![
            spawn(event.start()),
            spawn(signal.start()),
            spawn(recorder.start()),
            spawn(date.start()),
            spawn(alert.start()),
        ]);

        Ok(Server {
            config_file,
            config,
            running,
            timeout,
            window,
            sig_handle,
            clients,
        })
    }

    pub fn is_running(&self) -> bool {
        self.running.load()
    }

    pub fn close(&self) -> ServerResult<()> {
        self.timeout.notify_all();
        let (conn, _) = xcb::Connection::connect(None).unwrap();
        let event = xcb::ClientMessageEvent::new(
            32,
            self.window,
            xcb::ATOM_ANY,
            xcb::ClientMessageData::from_data32([0; 5]),
        );
        xcb::send_event_checked(
            &conn,
            false,
            self.window,
            xcb::EVENT_MASK_PROPERTY_CHANGE,
            &event,
        );
        conn.flush();
        if !self.sig_handle.is_closed() {
            self.sig_handle.close();
        }
        println!("Close");
        Ok(())
    }
}

fn check_clients(clients_result: Result<Vec<ClientResult<()>>, JoinError>) -> ServerResult<()> {
    let clients = clients_result?;
    for c in clients.into_iter() {
        c?;
    }
    Ok(())
}

impl Future for Server {
    type Output = ServerResult<()>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.is_running() {
            if let Poll::Ready(_) = self.clients.poll_unpin(cx) {
                self.sig_handle.close();
            }
            Poll::Pending
        } else {
            let closed = self.close();
            if closed.is_err() {
                return Poll::Ready(closed);
            }

            loop {
                if let Poll::Ready(c) = self.clients.poll_unpin(cx) {
                    return Poll::Ready(check_clients(c));
                }
            }
        }
    }
}

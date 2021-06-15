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

use tokio::spawn;
use tokio::task::{JoinError, JoinHandle};

use signal_hook_tokio::Handle;

use thiserror::Error;

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

type ServerResult<T> = Result<T, ServerError>;
type ClientThread = JoinHandle<Result<(), ClientError>>;

pub struct Server {
    config_file: Option<String>,
    config: Config,
    running: Running,
    timeout: Timeout,
    sig_handle: Handle,
    clients: Vec<ClientThread>,
}

impl Server {
    pub fn new(config_file: Option<String>) -> ServerResult<Server> {
        let config = Config::new(config_file.clone())?;
        let share = config.share()?;
        let a_conf = config.alert_config();
        let e_conf = config.event_config();
        let d_conf = config.date_config();
        let r_conf = config.recorder_config();

        let running = Running::new(true);
        let timeout = Timeout::new();

        let pid = Pid::new();
        let prod = Productive::new(false);
        let alerts_on = Running::new(true);

        let event = EventHandler::new(e_conf, pid.0.clone(), running.clone(), timeout.clone())?;
        let signal = SignalHandler::new(running.clone(), timeout.clone())?;
        let recorder = Recorder::new(share, r_conf, pid.1, running.clone(), prod.clone())?;
        let date = DateChecker::new(d_conf, running.clone(), alerts_on.clone(), timeout.clone())?;
        let alert = Alerter::new(a_conf, running.clone(), alerts_on, prod)?;

        let sig_handle = signal.handle();
        let clients = vec![
            spawn(event.start()),
            spawn(signal.start()),
            spawn(recorder.start()),
            spawn(date.start()),
            spawn(alert.start()),
        ];

        Ok(Server {
            config_file,
            config,
            running,
            timeout,
            sig_handle,
            clients,
        })
    }
}

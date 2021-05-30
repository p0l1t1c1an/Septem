mod alert;
mod client;
mod date_checker;
mod event_handler;
mod recorder;
mod signal_handler;

use crate::config::{Config, ConfigError};

use alert::{AlertError, Alerter};
use client::{Client, ClientError, Pid, Productive, Running, Timeout};
use date_checker::{*, StartStopTimes::*};
use event_handler::{EventError, EventHandler};
use recorder::{Recorder, RecorderError};
use signal_handler::{SignalError, SignalHandler};

use std::mem::replace;

use futures::future::{select, try_join_all, Either};
use tokio::spawn;
use tokio::task::{JoinError, JoinHandle};

use signal_hook_tokio::Handle;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Invalid Server State")]
    ServerStateError,

    #[error("{0}")]
    JoinAllError(#[from] JoinError),

    #[error("{0}")]
    RunningClientError(#[from] ClientError),

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
type Waiting = JoinHandle<Option<bool>>;

enum SeverState {
    Init(Alerter, EventHandler, Recorder),
    Running(Vec<ClientThread>, Waiting),
    Selected,
    Stopped(Waiting),
}

pub struct Server {
    config_file: Option<String>,
    config: Config,
    running: Running,
    event_time: Timeout,
    stop_time: Timeout,
    sig_handle: Handle,
    sig_thread: ClientThread,
    state: SeverState,
}

impl Server {
    pub fn new(config_file: Option<String>) -> ServerResult<Server> {
        let config = Config::new(config_file)?;
        let share  = config.share()?;
        let a_conf = config.alert_config();
        let e_conf = config.event_config();
        let d_conf = config.date_config();
        let r_conf = config.recorder_config(); 

        date_checker::sanity_check(&d_conf)?;
        
        let running = Running::new(true);
        let event_time = Timeout::new();
        let stop_time = Timeout::new();

        let pid = Pid::new();
        let prod = Productive::new(false);

        let event = EventHandler::new(e_conf, pid.0.clone(), running.clone(), event_time.clone())?;
        let signal = SignalHandler::new(running.clone(), event_time.clone(), stop_time.clone())?;
        let recorder = Recorder::new(share, r_conf, pid.1, running.clone(), prod.clone())?;
        let alert = Alerter::new(a_conf, running.clone(), prod)?;

        Ok(
            Server {
                config_file,
                config,
                running,
                event_time,
                stop_time,
                sig_handle: signal.handle(),
                sig_thread: spawn(signal.start()),
                state: SeverState::Init(alert, event, recorder),
            }
        )
    }

    async fn start(&mut self, next: StartStopTimes) -> ServerResult<()> {
        /*
         * Check state of next 
         * Should be EndOfMonitoring || EndOfDay w/ is_on true
         * Otherwise Error 
         * Then spawn wait thread waiting until duration in next 
         */

        let wait = spawn(self.stop_time.wait_timeout(d));
        let init = replace(&mut self.state, SeverState::Running(Vec::new(), wait));

        if let SeverState::Init(a, e, r) = init {
            let mut clients = vec![
                spawn(a.start()),
                spawn(e.start()),
                spawn(r.start()),
            ];
            if let SeverState::Running(ref mut c, ref _w) = self.state {
                c.append(&mut clients);
                Ok(())
            } else {
                Err(ServerError::ServerStateError)
            }
        } else {
            Err(ServerError::ServerStateError)
        }
    }

    async fn stop(&mut self) -> ServerResult<()> {
        /*
         * set up next wait for stopped state 
         * replace state to stopped 
         * check if in selected state
         * set running false and notify event time
         * Wait for clients to close
         */

        Ok(())
    }

    async fn restart(&mut self) -> ServerResult<()> {
        /*
         * Reload config from file
         * Load new pid and prod
         * Load new clients minus signal handler
         * Run clients and set up next wait time
         * Replace server state to running
         *
         */

        Ok(())
    }


    pub async fn run(mut self) -> ServerResult<()> { 

        // Wait until start of monitoring or is currently running
        let mut next = next_time(self.config.date_config()).await;
        loop {
            match next {
                EndOfDay(d, is_on) => {
                    if !is_on {
                        self.stop_time.wait_timeout(d).await;
                    } else {
                        break;
                    }
                }
                EndOfMonitoring(_) => {
                    break;
                }
                StartOfMonitoring(d) => {
                    self.stop_time.wait_timeout(d).await;
                }
            }
            next = next_time(self.config.date_config()).await;
        }


        /*
         * Loop with match case that should be checking 
         * what state we are in (simple state matchine)
         * Init simply runs start
         * 
         * Running will run a select between a join all 
         * of clients and the stop timeout wait 
         * for the contained Duration
         *
         * Stopped will simply use the stop_timeout 
         * to wait until next start returned.
         * (Another loop)
         * Then run restart
         *
         * Selected should simply run stop
         *
         * Also we should be checking if the singal handler
         * has been closed. If so we need to shut everything 
         * down and wait for the clients to close
         *
         */

        Ok(())
    }
}


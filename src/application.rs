mod alert;
mod client;
mod date_checker;
mod event_handler;
mod recorder;
mod signal_handler;

use crate::config::{date_config::DateTimeConfig, Config, ConfigError};

use alert::{AlertError, Alerter};
use client::{Client, ClientError, Pid, Productive, Running, Timeout};
use date_checker::DateError;
use event_handler::{EventError, EventHandler};
use recorder::{Recorder, RecorderError};
use signal_handler::{SignalError, SignalHandler};

use futures::future::{select, try_join_all, Either};
use tokio::spawn;
use tokio::task::{JoinError, JoinHandle};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
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

type AppResult<T> = Result<T, AppError>;

type ClientThread = JoinHandle<Result<(), ClientError>>;

/*
 *  TODO
 *
 *  Create a total server type that stores all clients,
 *  all way to stop and start all spawned threads.
 *  Plus new function to create that takes config file string
 *
 */

fn restart(
    running: &Running,
    time: &Timeout,
) -> AppResult<(DateTimeConfig, Vec<ClientThread>)> {
    let (share, rec_conf, event_conf, date_conf, alert_conf) = Config::new(None)?.break_up()?;
    date_checker::sanity_check(&date_conf)?;

    running.store(true);

    let pid = Pid::new();
    let prod = Productive::new(false);

    let event = EventHandler::new(event_conf, pid.0.clone(), running.clone(), time.clone())?;

    let recorder = Recorder::new(share, rec_conf, pid.1, running.clone(), prod.clone())?;
    let alert = Alerter::new(alert_conf, running.clone(), prod)?;

    let clients = vec![
        spawn(event.start()),
        spawn(recorder.start()),
        spawn(alert.start()),
    ];

    Ok((date_conf, clients))
}

pub async fn start() -> AppResult<()> {
    let running = Running::new(true);
    let cond = Condition::new();
    let time = Timeout::new();

    let (_, _, temp_date, _) = Config::new(None)?.break_up()?;
    date_checker::wait_next_start(temp_date).await;

    let signal = SignalHandler::new(running.clone(), cond.clone(), time.clone())?;
    let sig_thread = spawn(signal.start()); 

    let (mut date_conf, mut clients) = restart(&running, &cond)?;

    let mut joined = spawn(try_join_all(clients));
    let mut next = spawn(date_checker::wait_next(date_conf.clone(), time.clone()));

    'main: loop {
        match select(joined, next).await {
            Either::Left((j, _)) => {
                for error in j??.into_iter() {
                    error?;
                }
                break;
            }
            Either::Right((n, j)) => {
                if !running.load() {
                    break;
                }
                
                if let Some(start) = n? {
                    if start {
                        joined = j;
                        next = spawn(date_checker::wait_next(date_conf.clone(), time.clone()));
                    } else {
                        running.store(false);
                        cond.notify_one();

                        for error in j.await??.into_iter() {
                            error?;
                        }

                        'next: loop {
                           if let Some(is_on) = date_checker::wait_next(date_conf.clone(), time.clone()).await {
                                if is_on {
                                    break 'next;
                                }
                           } else {
                                break 'main;
                           }
                        }
                        
                        running.store(true);
                        let reset = restart(&running, &cond)?;

                        date_conf = reset.0;
                        clients = reset.1;

                        joined = spawn(try_join_all(clients));
                        next = spawn(date_checker::wait_next(date_conf.clone(), time.clone()));
                    }
                } else {
                    break;
                }
            }
        }
    }
    
    sig_thread.await??;
    println!("App End");
    Ok(())
}

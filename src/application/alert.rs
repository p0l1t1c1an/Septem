use crate::application::client::{Client, ClientResult, Shutdown};
use crate::application::config::AlertConfig;

use std::sync::atomic::Ordering;
use crossbeam::channel::{Receiver, TryRecvError};

use std::time::Duration;
use tokio::time::sleep;

use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AlertError {
    #[error("The alert message is empty and would not show anything")]
    EmptyMessageError,

    #[error("The sender was dropped while the receiver was up")]
    ReceiverError,
}

pub type AlertResult<T> = Result<T, AlertError>;

pub struct Alerter {
    is_prod: Receiver<bool>,
    shutdown: Shutdown,
    config: AlertConfig,
    productive: u64,
    unproductive: u64,
}

impl Alerter {
    fn sanity_check_conf(conf: &AlertConfig) -> AlertResult<()> {
        if conf.message().is_empty() {
            Err(AlertError::EmptyMessageError)
        } else {
            Ok(())
        }
    }

    pub fn new(conf: AlertConfig, shutdown: Shutdown, is_prod: Receiver<bool>) -> AlertResult<Alerter> {
        Alerter::sanity_check_conf(&conf)?;
        Ok(Alerter {
            is_prod: is_prod,
            shutdown: shutdown,
            config: conf,
            productive: 0,
            unproductive: 0,
        })
    }
}

#[async_trait]
impl Client for Alerter {
    async fn start(mut self) -> ClientResult {
        while !self.shutdown.load(Ordering::SeqCst) {
            match self.is_prod.try_recv() {
                Ok(p) => {
                    if p { 
                        self.productive += self.config.delay();
                    }
                    else {
                        self.unproductive += self.config.delay();
                    }
                    
                    if self.productive >= self.config.productive_time() * 60 {
                        self.productive = 0;
                        self.unproductive = 0;
                    }
                    else if self.unproductive >= self.config.unproductive_time() * 60 {
                        self.productive = 0;
                        self.unproductive = 0;
                        println!("{}", self.config.message());
                    }
                }
                Err(e) => match e {
                    TryRecvError::Empty => { }
                    TryRecvError::Disconnected => Err(AlertError::ReceiverError)?,
                }
            }
            
            sleep(Duration::from_secs(self.config.delay())).await;
                    
        }
        Ok(())
    }
}

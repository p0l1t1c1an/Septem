use crate::config::alert_config::AlertConfig;
use crate::server::client::{Client, ClientResult, Productive, Running};

use std::time::Duration;
use tokio::time::sleep;

use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AlertError {
    #[error("The alert message is empty and would not show anything")]
    EmptyMessageError,
    //#[error("The sender was dropped while the receiver was up")]
    //ReceiverError,
}

pub type AlertResult<T> = Result<T, AlertError>;

pub struct Alerter {
    running: Running,
    alerts_on: Running,
    is_prod: Productive,
    config: AlertConfig,
    productive: f64,
    unproductive: f64,
}

impl Alerter {
    fn sanity_check_conf(conf: &AlertConfig) -> AlertResult<()> {
        if conf.message().is_empty() {
            Err(AlertError::EmptyMessageError)
        } else {
            Ok(())
        }
    }

    pub fn new(
        config: AlertConfig,
        running: Running,
        alerts_on: Running,
        is_prod: Productive,
    ) -> AlertResult<Alerter> {
        Alerter::sanity_check_conf(&config)?;
        Ok(Alerter {
            running,
            alerts_on,
            is_prod,
            config,
            productive: 0.0,
            unproductive: 0.0,
        })
    }
}

#[async_trait]
impl Client for Alerter {
    async fn start(mut self) -> ClientResult<()> {
        while self.running.load() {
            sleep(Duration::from_millis(self.config.delay())).await;
            if self.alerts_on.load() {
                if self.is_prod.load() {
                    self.productive += self.config.delay() as f64 / 1000.0;
                    if self.productive >= self.config.productive_time() * 60.0 {
                        self.productive = 0.0;
                        self.unproductive = 0.0;
                    }
                } else {
                    self.unproductive += self.config.delay() as f64 / 1000.0;
                    if self.unproductive >= self.config.unproductive_time() * 60.0 {
                        self.productive = 0.0;
                        self.unproductive = 0.0;
                        println!("{}", self.config.message());
                    }
                }
            }
        }

        Ok(())
    }
}

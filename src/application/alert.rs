use crate::application::client::{Client, ClientResult};
use crate::application::config::AlertConfig;

use tokio::sync::mpsc::Receiver;

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
    is_prod: Receiver<(bool, u64)>,
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

    pub fn new(config: AlertConfig, is_prod: Receiver<(bool, u64)>) -> AlertResult<Alerter> {
        Alerter::sanity_check_conf(&config)?;
        Ok(Alerter {
            is_prod,
            config,
            productive: 0.0,
            unproductive: 0.0,
        })
    }
}

#[async_trait]
impl Client for Alerter {
    async fn start(mut self) -> ClientResult {
        while let Some((prod, time)) = self.is_prod.recv().await {
            if prod {
                self.productive += time as f64 / 1000.0;
                if self.productive >= self.config.productive_time() * 60.0 {
                    self.productive = 0.0;
                    self.unproductive = 0.0;
                }
            } else {
                self.unproductive += time as f64 / 1000.0;
                if self.unproductive >= self.config.unproductive_time() * 60.0 {
                    self.productive = 0.0;
                    self.unproductive = 0.0;
                    println!("{}", self.config.message());
                }
            }
        }

        Ok(())
    }
}

use crate::application::config::AlertConfig;

use std::sync::{atomic::AtomicBool, Arc, Condvar, Mutex};
use std::time::SystemTime;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AlertError {
    #[error("The alert message is empty and would not show anything")]
    EmptyMessageError,

    #[error("The {0} mutex failed to lock")]
    PosionedMutexError(String),

    #[error("The {0} condvar failed to load")]
    PosionedCondvarError(String),
}

pub type AlertResult<T> = Result<T, AlertError>;

pub struct Alerter {
    config: AlertConfig,
    start_time: SystemTime,
}

impl Alerter {
    fn sanity_check_conf(conf: &AlertConfig) -> AlertResult<()> {
        if conf.message().is_empty() {
            Err(AlertError::EmptyMessageError)
        } else {
            Ok(())
        }
    }

    pub fn new(conf: AlertConfig) -> AlertResult<Alerter> {
        Alerter::sanity_check_conf(&conf)?;
        Ok(Alerter {
            config: conf,
            start_time: SystemTime::now(),
        })
    }

    pub async fn start(
        self,
        productive: Arc<(Mutex<bool>, Condvar)>,
        shutdown: Arc<AtomicBool>,
    ) -> AlertResult<()> {
        Ok(())
    }
}

use crate::application::config::RecorderConfig;
use crate::application::process;

use process::{Process, ProcessError};
use tokio::task::{JoinError, JoinHandle};

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::time::SystemTime;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    },
};

use csv::{ReaderBuilder, WriterBuilder};
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

const DATA_FILE: &'static str = "data.csv";

#[derive(Error, Debug)]
pub enum RecorderError {
    #[error("Recorder error getting the new process:\n{0}")]
    GetProcessError(#[from] ProcessError),

    #[error("Share path {0} does not exist")]
    PathDoesNotExistError(String),

    #[error("{0}")]
    CsvError(#[from] csv::Error),

    #[error("{0}")]
    FileError(#[from] std::io::Error),

    #[error("{0}")]
    WriteThreadError(#[from] JoinError),

    #[error("The {0} mutex failed to lock")]
    PosionedMutexError(String),

    #[error("The {0} condvar failed to load")]
    PosionedCondvarError(String),
}

pub type RecorderResult<T> = Result<T, RecorderError>;

// Data type to be broken up as Recorder's hashmap
// and stored within the data file
#[derive(Debug, Deserialize, Serialize)]
struct Data {
    process_name: String,
    time_focused: u64,
    is_productive: bool,
}

pub struct Recorder {
    config: RecorderConfig,
    share_dir: String,
    prev_proc: Option<Process>,
    curr_proc: Option<Process>,
    start_time: SystemTime,
    write_time: SystemTime,
    proc_times: HashMap<String, (u64, bool)>,
}

impl Recorder {
    // Procedural Functions

    fn add_data(&mut self, data: Data) {
        match self.proc_times.get_mut(&data.process_name) {
            Some((t, p)) => {
                *t += data.time_focused;
                *p = data.is_productive;
            }
            None => {
                self.proc_times
                    .insert(data.process_name, (data.time_focused, data.is_productive));
            }
        }
    }

    fn create_data(path: &Path) -> RecorderResult<()> {
        if path.is_dir() {
            let data = path.join(DATA_FILE);
            let mut f = File::create(data)?;
            f.write_all(b"process_name,time_focused,is_productive")?;
            Ok(())
        } else {
            Err(RecorderError::PathDoesNotExistError(
                path.to_string_lossy().into_owned(),
            ))
        }
    }

    fn parse_data(
        share: &String,
        productive: &Vec<String>,
    ) -> RecorderResult<HashMap<String, (u64, bool)>> {
        let path = Path::new(share);
        let data = path.join(DATA_FILE);

        if data.exists() {
            let reader = ReaderBuilder::new().from_path(data)?;
            let mut map = HashMap::new();
            for r in reader.into_deserialize() {
                let data: Data = r?;
                let prod = productive.contains(&data.process_name);
                map.insert(data.process_name, (data.time_focused, prod));
            }
            Ok(map)
        } else {
            Recorder::create_data(path)?;
            Ok(HashMap::new())
        }
    }

    pub fn new(share: String, conf: RecorderConfig) -> RecorderResult<Recorder> {
        let map = Recorder::parse_data(&share, conf.productive())?;

        Ok(Recorder {
            config: conf,
            share_dir: share.to_owned(),
            prev_proc: None,
            curr_proc: None,
            start_time: SystemTime::now(),
            write_time: SystemTime::now(),
            proc_times: map,
        })
    }

    // Async Functions

    async fn write_data(
        share: String,
        proc_times: HashMap<String, (u64, bool)>,
    ) -> RecorderResult<()> {
        let mut writer = WriterBuilder::new().from_path(Path::new(&share).join(DATA_FILE))?;
        for (name, (time, prod)) in proc_times.into_iter() {
            writer.serialize(Data {
                process_name: name,
                time_focused: time,
                is_productive: prod,
            })?;
        }
        Ok(())
    }

    async fn wait_to_write(
        is_running: Option<JoinHandle<RecorderResult<()>>>,
        share: String,
        proc_times: HashMap<String, (u64, bool)>,
    ) -> RecorderResult<()> {
        if let Some(h) = is_running {
            h.await??;
        }
        Recorder::write_data(share, proc_times).await?;
        Ok(())
    }

    async fn wait_for_event(
        &mut self,
        pid: &Mutex<Option<u32>>,
        cond: &Condvar,
    ) -> RecorderResult<()> {
        match pid.lock() {
            Ok(p) => match cond.wait(p) {
                Ok(p) => {
                    self.prev_proc = self.curr_proc.clone();
                    self.curr_proc = match *p {
                        Some(u_val) => Some(Process::new(u_val as i32)?),
                        None => None,
                    }
                }
                Err(_) => Err(RecorderError::PosionedCondvarError("pid".to_owned()))?,
            },
            Err(_) => Err(RecorderError::PosionedMutexError("pid".to_owned()))?,
        }
        Ok(())
    }

    pub async fn start(
        mut self,
        pid_cond: Arc<(Mutex<Option<u32>>, Condvar)>,
        shutdown: Arc<AtomicBool>,
    ) -> RecorderResult<()> {
        let mut write_handle = tokio::spawn(Recorder::wait_to_write(
            None,
            self.share_dir.to_owned(),
            self.proc_times.to_owned(),
        ));
        self.write_time = SystemTime::now();

        while !shutdown.load(Ordering::SeqCst) {
            let (pid, cond) = &*pid_cond;
            self.wait_for_event(pid, cond).await?;

            if let Some(p) = self.prev_proc.clone() {
                let prod = self.config.productive().contains(&p.name);
                self.add_data(Data {
                    process_name: p.name,
                    time_focused: self.start_time.elapsed().unwrap().as_secs(),
                    is_productive: prod,
                });

                let write_elapsed = self.write_time.elapsed().unwrap().as_secs();
                self.write_time = SystemTime::now();

                if write_elapsed >= 30 {
                    write_handle = tokio::spawn(Recorder::wait_to_write(
                        Some(write_handle),
                        self.share_dir.to_owned(),
                        self.proc_times.to_owned(),
                    ));
                }
            }

            for (proc, (time, prod)) in &self.proc_times {
                println!("{}, {}, {}", proc, time, prod);
            }

            self.start_time = SystemTime::now();
        }

        write_handle.await??;
        Ok(())
    }
}

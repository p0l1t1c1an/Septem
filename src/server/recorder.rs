mod process;
use process::{Process, ProcessError};

use crate::config::recorder_config::RecorderConfig;
use crate::server::client::{Client, ClientResult, PidRecv, Productive, Running};

use tokio::task::JoinError;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::time::SystemTime;

use async_trait::async_trait;
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const DATA_FILE: &str = "data.csv";

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

    #[error("Pid channel closed")]
    PidChannelError,
}

pub type RecorderResult<T> = Result<T, RecorderError>;

// Data type to be broken up as Recorder's hashmap
// and stored within the data file
#[derive(Debug, Deserialize, Serialize)]
struct Data {
    name: String,
    time: u64,
    is_prod: bool,
}

pub struct Recorder {
    recv: PidRecv,
    running: Running,
    is_prod: Productive,
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
        match self.proc_times.get_mut(&data.name) {
            Some((t, p)) => {
                *t += data.time;
                *p = data.is_prod;
            }
            None => {
                self.proc_times.insert(data.name, (data.time, data.is_prod));
            }
        }
    }

    fn create_data(path: &Path) -> RecorderResult<()> {
        if path.is_dir() {
            let data = path.join(DATA_FILE);
            let mut f = File::create(data)?;
            f.write_all(b"name,time,is_prod")?;
            Ok(())
        } else {
            Err(RecorderError::PathDoesNotExistError(
                path.to_string_lossy().into_owned(),
            ))
        }
    }

    fn parse_data(
        share: &str,
        productive: &[String],
    ) -> RecorderResult<HashMap<String, (u64, bool)>> {
        let path = Path::new(share);
        let data = path.join(DATA_FILE);

        if data.exists() {
            let reader = ReaderBuilder::new().from_path(data)?;
            let mut map = HashMap::new();
            for r in reader.into_deserialize() {
                let data: Data = r?;
                let prod = productive.contains(&data.name);
                map.insert(data.name, (data.time, prod));
            }
            Ok(map)
        } else {
            Recorder::create_data(path)?;
            Ok(HashMap::new())
        }
    }

    pub fn new(
        share: String,
        conf: RecorderConfig,
        recv: PidRecv,
        running: Running,
        is_prod: Productive,
    ) -> RecorderResult<Recorder> {
        let map = Recorder::parse_data(&share, conf.productive())?;

        Ok(Recorder {
            recv,
            running,
            is_prod,
            config: conf,
            share_dir: share,
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
        for (name, (time, is_prod)) in proc_times.into_iter() {
            writer.serialize(Data {
                name,
                time,
                is_prod,
            })?;
        }
        Ok(())
    }

    async fn wait_for_event(&mut self) -> RecorderResult<()> {
        let get_proc = |u| -> RecorderResult<Process> { Ok(Process::new(u as i32)?) };
        self.prev_proc = self.curr_proc.clone();
        println!("Getting pid");
        self.curr_proc = match self.recv.recv().await {
            Some(pid) => match pid {
                Some(u) => Some(get_proc(u)?),
                None => None,
            },
            None => {
                return Err(RecorderError::PidChannelError);
            }
        };
        println!("Get pid");
        Ok(())
    }
}

#[async_trait]
impl Client for Recorder {
    async fn start(mut self) -> ClientResult<()> {
        let mut write_handle = tokio::spawn(Recorder::write_data(
            self.share_dir.to_owned(),
            self.proc_times.to_owned(),
        ));
        self.write_time = SystemTime::now();

        while self.running.load() {
            let error = self.wait_for_event().await;
            if let Err(e) = error {
                if let RecorderError::PidChannelError = e {
                    break;
                } else {
                    return Err(e.into());
                }
            }

            if let Some(p) = self.curr_proc.clone() {
                self.is_prod
                    .store(self.config.productive().contains(&p.name));
            } else {
                self.is_prod.store(false);
            }

            if let Some(p) = self.prev_proc.clone() {
                self.add_data(Data {
                    is_prod: self.config.productive().contains(&p.name),
                    name: p.name,
                    time: self.start_time.elapsed().unwrap().as_secs(),
                });

                let write_elapsed = self.write_time.elapsed().unwrap().as_secs();

                if write_elapsed >= self.config.write_delay() {
                    write_handle.await??;
                    write_handle = tokio::spawn(Recorder::write_data(
                        self.share_dir.to_owned(),
                        self.proc_times.to_owned(),
                    ));
                    self.write_time = SystemTime::now();
                }
            }

            println!("Data:");
            for (proc, (time, prod)) in &self.proc_times {
                println!("{}, {}, {}", proc, time, prod);
            }

            self.start_time = SystemTime::now();
        }

        write_handle.await??;
        println!("Rec End");
        Ok(())
    }
}

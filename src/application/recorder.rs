#[allow(dead_code)]
#[allow(unused_variables)]
use crate::application::process;
use process::{Process, ProcessError};

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::time::SystemTime;

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
}

pub type RecorderResult<T> = Result<T, RecorderError>;

// Data type to be stored in Recorder's hashmap
// and broken into the data file
#[derive(Debug, Deserialize, Serialize)]
struct Data {
    process_name: String,
    time_focused: u64,
}

pub struct Recorder {
    share_dir: String,
    curr_proc: Process,
    start_time: SystemTime,
    proc_times: HashMap<String, u64>,
}

// These should be an Arc of a Process and Arc of a SystemTime.
// They will be stored in application's startup process.
// Then, ewmh thread can update PID and the Arcs get reset
// in both the Recorder and Alert threads.

impl Recorder {
    fn create_date(path: &Path) -> RecorderResult<()> {
        if path.is_dir() {
            let data = path.join(DATA_FILE);
            File::create(data);
            Ok(())
        } else {
            Err(RecorderError::PathDoesNotExistError(
                path.to_string_lossy().into_owned(),
            ))
        }
    }

    fn parse_data(share: &String) -> RecorderResult<HashMap<String, u64>> {
        let path = Path::new(share);
        let data = path.join(DATA_FILE);
        if data.exists() {
            let reader = ReaderBuilder::new().from_path(data)?;
            let mut map = HashMap::new();
            for r in reader.into_deserialize() {
                let data: Data = r?;
                map.insert(data.process_name, data.time_focused);
            }
            Ok(map)
        } else {
            Recorder::create_date(path)?;
            Ok(HashMap::new())
        }
    }

    fn write_data(&self, share: &String) -> RecorderResult<()> {
        let mut writer = WriterBuilder::new().from_path(Path::new(share).join(DATA_FILE))?;
        for (name, time) in self.proc_times.clone().into_iter() {
            writer.serialize(Data {
                process_name: name,
                time_focused: time,
            })?;
        }
        Ok(())
    }

    // Will need to be thread that rw a data file
    // Then passed in atomic pid_t to generate a new Process when updated.
    // Need some way to poll when the variable changes. (Or a better method all-around)
    pub async fn new(share: String) -> RecorderResult<Recorder> {
        Ok(Recorder {
            share_dir: share.clone(),
            curr_proc: Process {
                pid: 0,
                name: String::new(),
            },
            start_time: SystemTime::now(),
            proc_times: Recorder::parse_data(&share)?,
        })
    }
}

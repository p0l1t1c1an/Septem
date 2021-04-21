
use crate::application::process;
use process::{Process, ProcessError};

use std::time::SystemTime;
use std::collections::HashMap;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecorderError {
    #[error("Recorder error getting the new process:\n{0}")]
    GetProcessError(#[from] ProcessError)
}

pub struct Recorder {
    share_dir : String,
    curr_proc : Process,   
    start_time : SystemTime,
    proc_times : HashMap<String, u64>,
}

// These should be an Arc of a Process and Arc of a SystemTime. 
// They will be stored in application's startup process. 
// Then, ewmh thread can update PID and the Arcs get reset 
// in both the Recorder and Alert threads.

impl Recorder {

    // Will need to be thread that rw a data file
    // Then passed in atomic pid_t to generate a new Process when updated.
    // Need some way to poll when the variable changes. (Or a better method all-around) 
    pub async fn new(share : String) -> Recorder {
        let proc_data = Recorder::parse_data(&share); 
    
        Recorder {
            share_dir : share,
            curr_proc : Process{pid : 0, name : String::new()},
            start_time : SystemTime::now(),
            proc_times : proc_data,
        }
    }

    fn parse_data(share : &String) -> HashMap<String, u64> {
       HashMap::new() 
    }

    pub async fn start(self) -> Result<(), RecorderError> {
        Ok(())
    }

}


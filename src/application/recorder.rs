
use crate::application::process;
use process::{Process, ProcessError};

use std::time::SystemTime;
use std::collections::HashMap;

use thiserror::Error;

#[derive(Error, Debug)]
enum RecorderError {
    #[error("Recorder Error getting the new process:\n{0}")]
    GetProcessError(#[from] ProcessError)
}

pub struct Recorder<'a> {
    share_dir : &'a String,
    curr_proc : Process,   
    start_time : SystemTime,
    proc_times : HashMap<String, u64>,
}

// These should be an Arc of a Process and Arc of a SystemTime. 
// They will be stored in application's startup process. 
// Then, ewmh thread can update PID and the Arcs get reset 
// in both the Recorder and Alert threads.

impl<'a> Recorder<'a> {

    // Will need to be thread that rw a data file
    // Then passed in atomic pid_t to generate a new Process when updated.
    // Need some way to poll when the variable changes. (Or a better method all-around) 
    pub fn new(share : &'a String) -> Recorder {         
    }

}


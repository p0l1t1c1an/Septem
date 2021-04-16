
use process::Process;

use std::time::SystemTime;
use std::collections::HashMap;

pub struct Recorder {
    share_dir : &String,
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
    pub fn spawn(share : &String) -> Recorder {         
    }

}


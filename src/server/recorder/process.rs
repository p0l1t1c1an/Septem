#![allow(unused_imports)]

use libc::{c_char, c_int, c_void, free, pid_t};

extern "C" {
    #[cfg(any(target_os = "freebsd", target_os = "openbsd"))]
    fn proc_name(name_ptr: *mut *mut c_char, pid: pid_t) -> c_int;
}

use std::ffi::CStr;
use std::fs::read_to_string;
use std::str::Utf8Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("Kinfo process name didn't exist for pid: {0}")]
    StrMallocError(pid_t),

    #[error("Kinfo failed to get infomation about pid: {0}")]
    KInfoError(pid_t),

    #[error("Failed to convert C str to Rust str")]
    AsciiToUtf8Error(#[from] Utf8Error),

    #[error("{0}")]
    FileProcError(#[from] std::io::Error),

    // Somehow the C code returns an unexpected value
    #[error("And you may ask yourself, hOw DiD I gET HeRE?")]
    UnknownError,
}

#[derive(Clone, Debug)]
pub struct Process {
    pub pid: pid_t,
    pub name: String,
}

unsafe impl Send for Process {}

impl Process {
    #[cfg(any(target_os = "freebsd", target_os = "openbsd"))]
    pub fn new(p: pid_t) -> Result<Process, ProcessError> {
        let mut proc = Process {
            pid: p,
            name: String::new(),
        };
        unsafe {
            let mut name_ptr: *mut c_char = std::ptr::null_mut();
            let err = proc_name(&mut name_ptr, p);
            match err {
                0 => proc.name.push_str(CStr::from_ptr(name_ptr).to_str()?),
                1 => return Err(ProcessError::StrMallocError(p)),
                2 => return Err(ProcessError::KInfoError(p)),
                _ => return Err(ProcessError::UnknownError),
            }
            free(name_ptr as *mut c_void);
        }

        Ok(proc)
    }

    #[cfg(target_os = "linux")]
    pub fn new(p: pid_t) -> Result<Process, ProcessError> {
        Ok(Process {
            pid: p,
            name: read_to_string(format!("/proc/{}/cmdline", p))?,
        })
    }
}


use libc::{c_int, c_char, pid_t, free, strlen};

extern "C" {
    fn proc_name(name_ptr : *mut c_char, pid : pid_t) -> c_int;
}

pub enum ProcessError {
    KInfoError,
    StrMallocError,
}

pub struct Process {
    pub pid : pid_t,
    pub name : String
}

impl Process {
    unsafe fn set_name(&mut self, ptr : *const c_char){
        for i in 0..strlen(ptr) {
            self.name.push(*ptr.offset(i as isize) as char);
        }
    }

    pub fn new(p : pid_t) -> Result<Process, ProcessError> {
        let proc = Process {
            pid : p,
            name : String::new()
        };
        unsafe {
            let name_ptr : *mut c_char;
            let err = proc_name(name_ptr, p);
            match err {
                0 => proc.set_name(name_ptr),
                1 => return Err(ProcessError::StrMallocError),
                2 => return Err(ProcessError::KInfoError),
            }
            free(name_ptr);
        }

        Ok(proc)

    }
}

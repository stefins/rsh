use std::{
    ffi::CString,
    sync::mpsc::{Receiver, Sender},
};

use libc::{chdir, kill, SIGINT};

// This function setup a thread to handle ctrl+c INT's
// The thread recieves a pid from a channel and send SIGINT to the pid
pub(crate) fn setup_interrupt_handler() -> Sender<i32> {
    use std::sync::mpsc;
    let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();
    ctrlc::set_handler(move || {
        if let Ok(pid) = rx.recv() {
            unsafe {
                kill(pid, SIGINT);
            };
        }
    })
    .expect("Cannot setup int handler!");
    tx
}

// Remove \n and \r from a line
pub(crate) fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

// wrapper around chdir syscall
pub(crate) fn change_dir(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let path = CString::new(path)?;
        chdir(path.as_ptr());
        Ok(())
    }
}

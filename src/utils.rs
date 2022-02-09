use std::sync::mpsc::{Receiver, Sender};

use libc::{kill, SIGINT};

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

pub(crate) fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

use std::{
    env,
    env::current_dir,
    ffi::CString,
    sync::mpsc::{Receiver, Sender},
};

use std::mem::MaybeUninit;
use std::sync::mpsc;

use libc::{
    chdir, kill, tcgetattr, tcsetattr, termios, ECHO, ICANON, ISIG, SIGINT, STDIN_FILENO, TCSAFLUSH,
};

// This function setup a thread to handle ctrl+c INT's
// The thread recieves a pid from a channel and send SIGINT to the pid
pub(crate) fn setup_interrupt_handler() -> Sender<i32> {
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
    env::set_var("OLDPWD", current_dir()?);
    let path = CString::new(path)?;
    unsafe {
        chdir(path.as_ptr());
    }
    env::set_var("PWD", current_dir()?);
    Ok(())
}

// Make the terminal to raw_mode
pub(crate) fn enable_raw_mode() -> MaybeUninit<termios> {
    let mut orig_termios: MaybeUninit<termios> = MaybeUninit::uninit();
    unsafe {
        let mut raw = orig_termios;
        tcgetattr(STDIN_FILENO, raw.as_mut_ptr());
        orig_termios = raw;
        (*raw.as_mut_ptr()).c_lflag &= !(ECHO | ICANON | ISIG);
        tcsetattr(STDIN_FILENO, TCSAFLUSH, raw.as_mut_ptr());
    }
    orig_termios
}

// Revert the terminal back to canonical mode
pub(crate) fn disable_raw_mode(mut orig_termios: MaybeUninit<termios>) {
    unsafe {
        tcsetattr(STDIN_FILENO, TCSAFLUSH, orig_termios.as_mut_ptr());
    }
}

use std::{
    env::current_dir,
    ffi::CString,
    fs::File,
    io::{BufReader, Read},
    os::unix::prelude::FromRawFd,
    str,
    sync::mpsc::{Receiver, Sender},
};

use std::mem::MaybeUninit;
use std::sync::mpsc;

use libc::{
    chdir, kill, tcgetattr, tcsetattr, termios, ECHO, ICANON, SIGINT, STDIN_FILENO, TCSAFLUSH,
};

use crate::set_env;
use crate::{flush, keyboard::Key};

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
pub(crate) fn change_dir(path: &str) -> Result<i32, Box<dyn std::error::Error>> {
    set_env!("OLDPWD", current_dir()?);
    let path = CString::new(path)?;
    unsafe {
        if chdir(path.as_ptr()) != 0 {
            println!("cd: no such directory: {}", &path.to_str()?);
            return Ok(1);
        }
    }
    set_env!("PWD", current_dir()?);
    Ok(0)
}

// Make the terminal to raw_mode
pub(crate) fn enable_raw_mode() -> MaybeUninit<termios> {
    let mut orig_termios: MaybeUninit<termios> = MaybeUninit::uninit();
    unsafe {
        let mut raw = orig_termios;
        tcgetattr(STDIN_FILENO, raw.as_mut_ptr());
        orig_termios = raw;
        (*raw.as_mut_ptr()).c_lflag &= !(ECHO | ICANON);
        tcsetattr(STDIN_FILENO, TCSAFLUSH, raw.as_mut_ptr());
    }
    orig_termios
}

// Revert the terminal back to canonical mode
pub(crate) fn disable_raw_mode(mut orig_termios: MaybeUninit<termios>) {
    unsafe {
        tcsetattr(STDIN_FILENO, TCSAFLUSH, orig_termios.as_mut_ptr());
    }
    flush!();
}

pub(crate) fn read_chars() -> Result<Key, Box<dyn std::error::Error>> {
    let f = unsafe { File::from_raw_fd(0) };
    let mut reader = BufReader::new(f);
    let mut buffer = [0; 3];
    reader.read(&mut buffer).unwrap();
    Ok(match str::from_utf8(&buffer).unwrap() {
        "\u{1b}[A" => Key::UpKey,
        "\u{1b}[B" => Key::DownKey,
        "\u{1b}[C" => Key::RightKey,
        "\u{1b}[D" => Key::LeftKey,
        _ => Key::OtherKey,
    })
}

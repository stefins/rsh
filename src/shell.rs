use std::io::BufRead;

use crate::flush;
use crate::set_env;
use crate::utils;
use std::env;
use std::io;
use std::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub struct Shell<'a> {
    pub path: String,
    chr: &'a str,
    pub interrupter: Sender<i32>,
}

impl<'a> Shell<'a> {
    pub(crate) fn new() -> Self {
        let path = match env::var("PATH") {
            Ok(path) => path,
            Err(_) => panic!("$PATH not found"),
        };
        set_env!("SHELL", env::current_exe().unwrap());
        let chr = "=>";
        let interrupter = utils::setup_interrupt_handler();
        Shell {
            path,
            chr,
            interrupter,
        }
    }

    pub(crate) fn listen(&self) {
        loop {
            print!("\r{} ", self.chr);
            flush!();
            let mut command = String::new();
            let stdin = io::stdin();
            let mut handle = stdin.lock();
            handle.read_line(&mut command).expect("EOF");
            crate::command::Command::new(self, command)
                .execute()
                .unwrap();
        }
    }
}

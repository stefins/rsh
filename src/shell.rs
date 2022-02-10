use std::io::BufRead;

use crate::utils;
use std::env;
use std::io;
use std::io::Write;
use std::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub struct Shell {
    pub path: String,
    chr: char,
    pub interrupter: Sender<i32>,
}

impl Shell {
    pub(crate) fn new() -> Self {
        let path = match env::var("PATH") {
            Ok(path) => path,
            Err(_) => panic!("$PATH not found"),
        };
        env::set_var("SHELL", env::current_exe().unwrap());
        let chr = '$';
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
            io::stdout().flush().unwrap();
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

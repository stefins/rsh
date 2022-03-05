use crate::flush;
use crate::keyboard::Key;
use crate::set_env;
use crate::utils;
use crate::utils::disable_raw_mode;
use crate::utils::enable_raw_mode;
use crate::utils::read_chars;
use std::env;
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
            flush!();
            let mut command = String::new();
            let default_mode = enable_raw_mode();
            loop {
                print!("\r{} {command}", self.chr);
                flush!();
                match read_chars() {
                    Ok((Key::Enter, None)) => {
                        println!();
                        break;
                    }
                    Ok((_, Some(ch))) => {
                        command.push(ch);
                    }
                    Ok((Key::Backspace, None)) => {
                        print!("\r{}", " ".repeat(command.len() + 3));
                        flush!();
                        command.pop();
                    }
                    Ok((_, None)) => {}
                    Err(_) => panic!("Error Occured"),
                }
            }
            disable_raw_mode(default_mode);
            crate::command::Command::new(self, command)
                .execute()
                .unwrap();
        }
    }
}

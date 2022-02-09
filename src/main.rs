use std::io::BufRead;
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};
use std::{
    env,
    io::{self, Write},
};

use libc::{kill, SIGINT};

fn main() {
    let shell = Shell::new();
    shell.listen();
}

#[derive(Clone, Debug)]
struct Shell {
    path: String,
    chr: char,
    interrupter: Sender<i32>,
}

impl Shell {
    fn new() -> Self {
        let path = match env::var("PATH") {
            Ok(path) => path,
            Err(_) => panic!("$PATH not found"),
        };
        let chr = '$';
        let interrupter = setup_interrupt_handler();
        Shell {
            path,
            chr,
            interrupter,
        }
    }

    pub fn listen(&self) {
        loop {
            print!("{} ", self.chr);
            io::stdout().flush().unwrap();
            let mut command = String::new();
            let stdin = io::stdin();
            let mut handle = stdin.lock();
            handle.read_line(&mut command).unwrap();
            Command::new(&self, command).execute().unwrap();
        }
    }
}

#[derive(Clone, Debug)]
struct Command<'a> {
    shell: &'a Shell,
    bin_path: String,
    pname: String,
    commands: Vec<String>,
    pid: u32,
}

impl<'a> Command<'a> {
    fn new(shell: &'a Shell, command: String) -> Self {
        let commands: Vec<String> = command.split(" ").map(|s| s.to_string()).collect();
        let mut bin_path = String::from("");
        let mut pname = commands[0].to_string();
        trim_newline(&mut pname);
        let paths: Vec<&str> = shell.path.split(":").collect();
        for pth in paths {
            let path = format!("{}/{}", pth, pname);
            if Path::new(&path).exists() {
                bin_path = path.to_string();
                break;
            }
        }
        Self {
            commands,
            pname,
            bin_path,
            shell,
            pid: 0,
        }
    }

    fn execute(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(last) = self.commands.last_mut() {
            trim_newline(last)
        };
        let child = std::process::Command::new(&self.bin_path)
            .args(&self.commands[1..])
            .spawn()
            .expect("Cannot execute command");
        self.pid = child.id();
        self.set_interrupt_handler();
        let output = child.wait_with_output()?;
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
        std::io::stdout().flush().unwrap();
        Ok(())
    }

    fn set_interrupt_handler(&self) {
        self.shell.interrupter.send(self.pid as i32).unwrap();
    }
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

fn setup_interrupt_handler() -> Sender<i32> {
    use std::sync::mpsc;
    let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();
    ctrlc::set_handler(move || {
        for pid in rx.recv() {
            unsafe {
                kill(pid, SIGINT);
            };
        }
    })
    .expect("Cannot setup int handler!");
    return tx;
}

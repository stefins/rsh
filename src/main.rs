use std::io::BufRead;
use std::path::Path;
use std::process::exit;
use std::{
    env,
    io::{self, Write},
};

use libc::{signal, SIGINT, kill};

fn main() {
    unsafe {
        signal(SIGINT, exit_handler as usize);
    }
    let shell = Shell::new();
    shell.listen();
}

fn exit_handler() {
    exit(0);
}

#[derive(Clone, Debug)]
struct Shell {
    path: String,
    chr: char,
}

impl Shell {
    fn new() -> Self {
        let path = match env::var("PATH") {
            Ok(path) => path,
            Err(_) => panic!("$PATH not found"),
        };
        let chr = '$';
        Shell { path, chr }
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
        let output = child.wait_with_output()?;
        self.set_interrupt_handler();
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
        std::io::stdout().flush().unwrap();
        Ok(())
    }

    fn set_interrupt_handler(&self) {
        unsafe {
            signal(SIGINT, interrupt as usize );
        }
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

fn interrupt (pid: i32) {
    unsafe {
        kill(pid, SIGINT);
    }
}

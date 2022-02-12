use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::exit;

use crate::shell::Shell;
use crate::{flush, utils};

#[derive(Clone, Debug)]
pub(crate) struct Command<'a> {
    shell: &'a Shell,
    bin_path: String,
    pname: String,
    commands: Vec<String>,
    pid: u32,
    available: bool,
    exit_code: i32,
}

impl<'a> Command<'a> {
    pub(crate) fn new(shell: &'a Shell, command: String) -> Self {
        let commands: Vec<String> = command.split(' ').map(|s| s.to_string()).collect();
        let mut bin_path = String::from("");
        let mut pname = commands[0].to_string();
        utils::trim_newline(&mut pname);
        let paths: Vec<&str> = shell.path.split(':').collect();
        let mut available = false;
        for pth in paths {
            let path = format!("{}/{}", pth, pname);
            if Path::new(&path).exists() {
                bin_path = path;
                available = true;
                break;
            }
        }
        Self {
            commands,
            pname,
            bin_path,
            shell,
            pid: 0,
            available,
            exit_code: 0,
        }
    }

    pub(crate) fn execute(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(last) = self.commands.last_mut() {
            utils::trim_newline(last)
        };
        if self.commands[0].is_empty() {
            exit(0);
        }
        if self.pname == "cd" {
            if self.commands.len() == 1 {
                utils::change_dir(&env::var("HOME")?)?;
            } else {
                utils::change_dir(&self.commands[1])?
            }
            return Ok(());
        }
        if self.pname == "exit" {
            if self.commands.len() == 1 {
                exit(0);
            } else {
                let exit_code: i32 = self.commands[1].parse()?;
                exit(exit_code);
            }
        }
        if !self.available {
            println!("command not found: {}", self.pname);
            return Ok(());
        }
        let child = std::process::Command::new(&self.bin_path)
            .args(&self.commands[1..])
            .spawn();
        let child = child?;
        self.pid = child.id();
        self.set_interrupt_handler();
        let output = child.wait_with_output()?;
        self.exit_code = output.status.code().unwrap();
        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)?;
        flush!();
        Ok(())
    }

    fn set_interrupt_handler(&self) {
        self.shell.interrupter.send(self.pid as i32).unwrap();
    }
}

#[cfg(test)]
mod tests {

    use std::{thread::sleep, time::Duration};

    use super::*;

    #[test]
    fn test_command_available() {
        let shell = Shell::new();
        let command1 = Command::new(&shell, String::from("true"));
        let command2 = Command::new(&shell, String::from("true000"));
        assert_eq!(command1.available, true);
        assert_eq!(command2.available, false);
    }

    #[test]
    fn test_exit_status() {
        sleep(Duration::from_secs(1));
        let shell = Shell::new();
        let mut command = Command::new(&shell, String::from("false"));
        command.execute().unwrap();
        assert_eq!(command.exit_code, 1);
    }
}

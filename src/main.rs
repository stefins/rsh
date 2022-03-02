mod command;
mod keyboard;
mod macros;
mod shell;
mod utils;
use shell::Shell;

#[cfg_attr(test, macro_use)]
extern crate lazy_static;

fn main() {
    let shell = Shell::new();
    shell.listen();
}

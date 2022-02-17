mod command;
mod keyboard;
mod macros;
mod shell;
mod utils;
use shell::Shell;

#[allow(unused_imports)]
#[macro_use]
extern crate lazy_static;

fn main() {
    let shell = Shell::new();
    shell.listen();
}

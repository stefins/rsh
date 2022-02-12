mod command;
mod keyboard;
mod macros;
mod shell;
mod utils;

use shell::Shell;

fn main() {
    let shell = Shell::new();
    shell.listen();
}

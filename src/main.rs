use clap::Parser;

use code_enter::command::{handle_command, Cli};

fn main() {
    let cli = Cli::parse();

    handle_command(&cli.command);
}

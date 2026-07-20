use clap::Parser;

use code_enter::command::{handle_command, Cli};

fn main() {
    let cli = Cli::parse();

    std::process::exit(handle_command(&cli.command));
}

use clap::{Parser, Subcommand, ValueEnum};

use crate::handle;

#[derive(Parser, Debug)]
#[command(author,version,about,long_about= None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

// enum of specific sub-commands
// display the name and the structure of needed parameters
#[derive(Subcommand, Debug)]
pub enum Commands {
    Add {
        alias: String,
        path: String,
    },
    Ed {
        alias: String,
        path: String,
    },
    Del {
        alias: String,
    },
    Jp {
        alias: String,
    },
    List,
    Init {
        shell: Shell,
        #[arg(long)]
        force: bool,
    },
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Shell {
    Powershell,
    Bash,
    Zsh,
}

pub fn handle_command(cmd: &Commands) -> i32 {
    // 处理子命令
    handle::handle(cmd)
}

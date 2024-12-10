pub mod subcommands;

use clap::{
    Parser,
    Subcommand
};
use crate::cli::subcommands::{
    admin::AdminCommand,
    config::ConfigCommand,
    run::RunCommand,
    daemon::DaemonCommand
};



#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct CauthCli {
    #[clap(subcommand)]
    pub action: ActionType
}

impl CauthCli {
    pub fn run(self) {
        let _ = match self.action {
            ActionType::Run(cmd) => cmd.run(),
            ActionType::Daemon(cmd) => cmd.run(),
            ActionType::Config(cmd) => cmd.run(),
            ActionType::Admin(cmd) => cmd.run()
        };
    }
}

#[derive(Debug, Subcommand)]
pub enum ActionType {
    Run(RunCommand),
    Daemon(DaemonCommand),
    Config(ConfigCommand),
    Admin(AdminCommand)
}

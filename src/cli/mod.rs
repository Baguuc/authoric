pub mod subcommands;

use clap::{
    Parser,
    Subcommand
};
use crate::{cli::subcommands::{
    admin::AdminCommand, config::ConfigCommand, daemon::DaemonCommand, run::RunCommand
}, config::CauthConfig};



#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct CauthCli {
    #[clap(subcommand)]
    pub action: ActionType
}

impl CauthCli {
    pub fn run(self, config: CauthConfig) {
        let _ = match self.action {
            ActionType::Run(cmd) => cmd.run(config),
            ActionType::Daemon(cmd) => cmd.run(config),
            ActionType::Config(cmd) => cmd.run(config),
            ActionType::Admin(cmd) => cmd.run(config)
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

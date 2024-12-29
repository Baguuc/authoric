use clap::{
  Args,
  Subcommand
};

use crate::config::CauthConfig;


#[derive(Debug, Args)]
pub struct ConfigCommand {
  #[clap(subcommand)]
  pub action: ConfigAction
}

#[derive(Debug, Subcommand)]
pub enum ConfigAction {
  Edit
}

impl ConfigCommand {
  pub fn run(self) {
    match self.action {
      ConfigAction::Edit => {
        CauthConfig::edit();
      }
    };
  }
}
use clap::Args;

use crate::config::CauthConfig;


#[derive(Debug, Args)]
pub struct RunCommand;


impl RunCommand {
  pub fn run(self, config: CauthConfig) {
    todo!()
  }
}
use clap::Args;

use crate::{config::CauthConfig, web::run_server};


#[derive(Debug, Args)]
pub struct RunCommand {

}


impl RunCommand {
  pub fn run(self, config: CauthConfig) {
    let _ = run_server(config.port);
  }
}
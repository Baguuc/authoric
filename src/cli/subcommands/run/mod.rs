use clap::Args;
use futures::executor::block_on;

use crate::{config::CauthConfig, web::run_server};


#[derive(Debug, Args)]
pub struct RunCommand {

}


impl RunCommand {
  pub fn run(self, config: CauthConfig) {
    let _ = block_on(run_server(config));
  }
}
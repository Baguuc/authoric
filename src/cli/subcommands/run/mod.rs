use crate::{cli::init_defaults, config::CauthConfig, web::run_server};
use clap::Args;
use futures::executor::block_on;

#[derive(Debug, Args)]
pub struct RunCommand;

impl RunCommand {
    pub fn run(self, config: CauthConfig) {
        block_on(init_defaults(&config));
        let _ = block_on(run_server(config));
    }
}

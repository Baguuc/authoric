use clap::Args;

use crate::config::CauthConfig;


#[derive(Debug, Args)]
pub struct DaemonCommand;

impl DaemonCommand {
    pub fn run(self, config: CauthConfig) {
        todo!()
    }
}
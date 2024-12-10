use clap::{
    Args,
    Subcommand
};


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
        todo!()
    }
}
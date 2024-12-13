use clap::Parser;
use cli::CauthCli;

mod cli;
mod config;
mod models;
mod util;

#[tokio::main]
async fn main() {
    let cli = CauthCli::parse();
    let _ = cli.run();
}
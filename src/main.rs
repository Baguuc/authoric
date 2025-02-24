use chrono::{offset::Utc, DateTime};
use clap::Parser;
use cli::CauthCli;

mod cli;
mod config;
mod models;
mod util;
mod web;

#[tokio::main]
async fn main() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            let system_time = std::time::SystemTime::now();
            let datetime: DateTime<Utc> = system_time.into();
            let datetime_str = datetime.format("%d-%m-%Y:%T");

            out.finish(format_args!(
                "[{} {} {}] {}",
                datetime_str,
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    let cli = CauthCli::parse();
    let _ = cli.run();
}

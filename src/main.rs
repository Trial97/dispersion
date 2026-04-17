use clap::Parser;
use eyre::Result;
use log::info;

mod cli;
mod logger;
mod self_updater;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    logger::init(&cli)?;
    let client = reqwest::blocking::Client::new();
    self_updater::update(&cli, &client)?;

    match cli.command {
        cli::Commands::Check => {
            info!("check command")
        }
        cli::Commands::Update => {
            info!("update command")
        }
    }
    Ok(())
}

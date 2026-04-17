use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Commands {
    Check,
    Update,
}

#[derive(Parser, Debug)]
#[command[version,about,long_about=None]]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    // logger config
    #[arg(
        long,
        help = "Path to the log file",
        value_name = "log file",
        default_value = "dispersion.log"
    )]
    pub log_path: PathBuf,

    #[arg(
        long,
        help = "Controls if the log should be printed to standard output"
    )]
    pub log_stdout: bool,

    #[arg(long, help = "The level of log to be logged", default_value = "debug")]
    pub log_level: log::LevelFilter,

    #[arg(long, help = "Updater repo owner", default_value = "Trial97")]
    pub updater_repo_owner: String,
    #[arg(long, help = "Updater repo name", default_value = "dispersion")]
    pub updater_repo_name: String,
    #[arg(long, help = "Updater public signature URL", default_value = "")]
    pub updater_signature_url: String,
}

use std::path::PathBuf;

use clap::Parser;

use crate::paths::get_exe_root_dir;

#[derive(Parser, Debug)]
#[command[version,about,long_about=None]]
pub struct Cli {
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

    // updater config
    #[arg(long, help = "Updater repo owner", default_value = "Trial97")]
    pub updater_repo_owner: String,
    #[arg(long, help = "Updater repo name", default_value = "dispersion")]
    pub updater_repo_name: String,
    #[arg(long, help = "Updater public signature URL", default_value = "")]
    pub updater_signature_url: String,

    // repo config
    #[arg(long, help = "Repo owner", default_value = "PrismLauncher")]
    pub repo_owner: String,
    #[arg(long, help = "Repo name", default_value = "PrismLauncher")]
    pub repo_name: String,
    #[arg(long, help = "binary name", default_value = "prismlauncher")]
    pub binary_name: String,
    #[arg(long, help = "Current version", default_value = "10.0.5")]
    pub current_version: String,

    #[arg(long, help = "root directory", default_value_os_t = get_exe_root_dir().unwrap_or_default())]
    pub root_dir: PathBuf,
    #[arg(long, help = "compiler type", default_value = "")]
    pub compiler: String,
}

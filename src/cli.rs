use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::github;

#[derive(Subcommand, Debug)]
pub enum Commands {
    Check,
    Update,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CommandArgs {
    // paths
    #[arg(
        long,
        help = "Path to the log file",
        value_name = "log file",
        default_value = "dispersion.log"
    )]
    pub log_path: PathBuf,

    #[arg(
        long,
        help = "Path to the root directory",
        value_name = "root directory",
        default_value = "."
    )]
    pub root_path: PathBuf,

    #[arg(
        long,
        help = "Path to the temporary directory",
        value_name = "tmp directory"
    )]
    pub tmp_path: Option<PathBuf>,

    // github
    #[arg(
        long,
        help = "GitHub repository owner name",
        value_name = "repo owner",
        default_value = "PrismLauncher"
    )]
    pub repo_owner: String,

    #[arg(
        long,
        help = "GitHub repository name",
        value_name = "repo name",
        default_value = "PrismLauncher"
    )]
    pub repo_name: String,

    // github action
    #[arg(
        long,
        help = "Workflow file name",
        value_name = "workflow",
        default_value = "trigger_builds.yml"
    )]
    pub workflow_name: String,

    #[arg(
        long,
        help = "Repository main branch",
        value_name = "branch",
        default_value = "develop"
    )]
    pub branch: String,

    // version
    #[arg(
        long,
        help = "Use this version as the installed launcher version",
        value_name = "version",
        required = true
    )]
    pub prism_version: Option<String>,

    #[arg(
        long,
        help = "Git commit hash associated with the build",
        value_name = "commit hash",
        required = true
    )]
    pub git_commit: Option<String>,

    #[arg(
        long,
        help = "The built artifact",
        value_name = "artifact",
        required = true
    )]
    pub build_artifact: Option<String>,

    #[arg(long, help = "The app binary name", value_name = "binary name")]
    pub app_name: Option<String>,

    // config
    #[arg(
        long,
        help = "Type of release",
        value_name = "type",
        default_value = "nightly"
    )]
    pub release_type: github::ReleaseType,

    #[arg(
        short = 'f',
        help = "Force an update, even if one is not needed",
        default_value = "false"
    )]
    pub force: bool,

    #[arg(long, help = "Should log be printed on std_out")]
    pub log_stdout: bool,

    #[arg(
        long,
        help = "Should log be printed on std_out",
        default_value = "debug"
    )]
    pub log_level: log::LevelFilter,

    #[command(subcommand)]
    pub command: Commands,
}

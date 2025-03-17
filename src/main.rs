use backup::{backup_app_dir, move_with_manifest};
use clap::Parser;
use file_lock::FileLock;
use install::{call_appimage_update, run_installer};
use std::fs::{self, create_dir};
use tempfile::tempdir;
use tokio::{self, process::Command};

use download::fetch_url;
use system::{
    InstallationType, compare_tags, get_exe_root_dir, get_instalation_type, select_valid_artifacts,
};
use unpack::unarchive_loop;

mod backup;
mod cli;
mod download;
mod file_lock;
mod github;
mod install;
mod system;
mod unpack;

fn init_log(args: &cli::CommandArgs) -> eyre::Result<()> {
    let mut log_cfg = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339(std::time::SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        // Add blanket level filter -
        .level(log::LevelFilter::Debug)
        // - and per-module overrides
        .level_for("hyper", log::LevelFilter::Info)
        // Output to stdout, files, and other Dispatch configurations
        .chain(fern::log_file(&args.log_path)?);
    if args.log_stdout {
        log_cfg = log_cfg.chain(std::io::stdout());
    }
    // Apply globally
    log_cfg.apply()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), eyre::Report> {
    let cli = cli::CommandArgs::parse();
    init_log(&cli)?;

    let build_artifact = match cli.build_artifact.as_deref() {
        Some("") | None => {
            log::error!("Error: build_artifact is missing or empty.");
            return Err(eyre::eyre!("build_artifact is missing or empty"));
        }
        Some(v) => v,
    };

    let release = match github::get_latest(&cli).await {
        Ok(release) => release,
        Err(err) => {
            log::error!("Failed to get latest release: {:?}", err);
            return Err(err);
        }
    };

    match cli.release_type {
        github::ReleaseType::Stable => match cli.prism_version.as_deref() {
            Some("") | None => {
                log::error!("Error: prism_version is missing or empty.");
                return Err(eyre::eyre!("prism_version is missing or empty"));
            }
            Some(actual_version) => {
                match compare_tags(release.tag.clone(), actual_version.to_owned()) {
                    Ok(false) => {
                        log::info!(
                            "Nothing to do current version is greater or equal to the latest release: {:?} vs {:?}",
                            release.tag,
                            actual_version
                        );
                        return Ok(());
                    }
                    Ok(true) => {}
                    Err(err) => {
                        log::error!("Failed to compare versions: {:?}", err);
                        return Err(err);
                    }
                }
            }
        },
        github::ReleaseType::Nightly => match cli.git_commit.as_deref() {
            Some("") | None => {
                log::error!("Error: git_commit is missing or empty.");
                return Err(eyre::eyre!("git_commit is missing or empty"));
            }
            Some(commit) => match commit == release.tag {
                true => {
                    log::info!(
                        "Nothing to do current version is the same as latest release: {:?} vs {:?}",
                        release.tag,
                        commit
                    );
                    return Ok(());
                }
                false => {}
            },
        },
    };
    let root_dir = if cli.root_path.exists() {
        cli.root_path
    } else {
        // fallback just in case
        match get_exe_root_dir() {
            Some(dir) => dir,
            None => {
                log::error!("Failed to retrieve root directory");
                return Err(eyre::eyre!("Missing root directory"));
            }
        }
    };
    let installation_type = get_instalation_type(&root_dir);
    let valid_artifacts = match select_valid_artifacts(
        &release,
        build_artifact.to_owned(),
        installation_type.clone(),
    ) {
        Ok(valid_artifacts) => valid_artifacts,
        Err(err) => {
            log::error!("Failed to filter artifacts: {:?}", err);
            return Err(err);
        }
    };
    let first_version = match valid_artifacts.first() {
        Some(v) => v,
        None => {
            log::error!("Valid artifacts empty?");
            return Err(eyre::eyre!("Valid artifacts empty"));
        }
    };
    log::info!("Valid Artifact:{:?}", first_version);

    match cli.command {
        cli::Commands::Check => {
            println!("Name: {}", release.name);
            println!("Version: {}", release.tag);
            println!("TimeStamp: {}", release.created_at.format("%+"));
            println!("{}", release.body.unwrap_or("".to_string()));
            std::process::exit(100);
        }
        cli::Commands::Update => {
            if installation_type == InstallationType::Appimage {
                match call_appimage_update(&root_dir).await {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("Failed to call appimage updater: {:?}", err);
                        return Err(err.into());
                    }
                };
            }
            let _lock = match FileLock::lock(root_dir.join("update.lock")) {
                Ok(v) => v,
                Err(err) => {
                    log::error!("Failed to lock updater: {:?}", err);
                    return Err(err.into());
                }
            };

            let temp_dir = match tempdir() {
                Ok(v) => v,
                Err(err) => {
                    log::error!("Failed to create temporary directory: {:?}", err);
                    return Err(err.into());
                }
            };
            let temp_dir_path = match cli.tmp_path {
                Some(v) => {
                    if v.exists() {
                        match fs::remove_dir_all(&v) {
                            Ok(_) => {}
                            Err(err) => {
                                log::error!("Failed to create temporary directory: {:?}", err);
                                return Err(err.into());
                            }
                        };
                    }
                    create_dir(&v)?;
                    v
                }
                None => temp_dir.path().into(),
            };

            let artifact_path =
                match fetch_url(first_version.download_url.clone(), &temp_dir_path).await {
                    Ok(v) => v,
                    Err(err) => {
                        log::error!("Failed to download artifact: {:?}", err);
                        return Err(err);
                    }
                };
            log::info!("downloaded to:{:?}", artifact_path);
            let final_path = match unarchive_loop(&artifact_path, &temp_dir_path) {
                Ok(v) => v, // here start the updater again
                Err(err) => {
                    log::info!("Nothing to unzip: {:?}", err);
                    artifact_path // execute this
                }
            };
            log::info!("unziped to:{:?}", final_path);

            if installation_type == InstallationType::Portable && final_path.is_dir() {
                match backup_app_dir(
                    &root_dir,
                    cli.prism_version.as_deref().unwrap(),
                    cli.git_commit.as_deref().unwrap(),
                    cli.build_artifact.as_deref().unwrap(),
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("Failed to backup: {:?}", err);
                        return Err(err);
                    }
                };
                match move_with_manifest(
                    &final_path,
                    &root_dir,
                    cli.build_artifact.unwrap().to_lowercase().contains("linux"),
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("Failed to copy files: {:?}", err);
                        return Err(err);
                    }
                }
                match cli.app_name {
                    None => {}
                    Some(mut app_name) => {
                        #[cfg(target_os = "windows")]
                        {
                            app_name += ".exe";
                        }
                        #[cfg(target_os = "linux")]
                        {
                            app_name = "bin/".to_owned() + &app_name;
                        }
                        let mut command = Command::new(app_name);
                        // Set the environment variable if on Windows
                        #[cfg(target_os = "windows")]
                        {
                            command.env("__COMPAT_LAYER", "RUNASINVOKER");
                        }
                        // Start the process detached
                        command.spawn()?;
                    }
                };
            } else {
                match run_installer(&final_path).await {
                    Ok(v) => {
                        std::process::exit(v.code().unwrap_or(1));
                    }
                    Err(err) => {
                        log::error!("Failed to run installer: {:?}", err);
                        return Err(err.into());
                    }
                };
            }
        }
    }

    Ok(())
}

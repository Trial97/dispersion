#[cfg(unix)]
use std::os::unix::process::CommandExt as _;
#[cfg(windows)]
use std::process::exit;

use eyre::{Result, eyre};
use log::error;
use log::info;
use self_update::cargo_crate_version;

use crate::cli;

fn prepare_keys(url: &String, client: &reqwest::blocking::Client) -> Vec<[u8; 32]> {
    let mut output: Vec<[u8; 32]> = vec![*include_bytes!("../keys/public.key")];

    if !url.is_empty() {
        match client
            .get(url)
            .send()
            .and_then(|res| res.bytes())
            .map_err(|e| eyre!(e))
            .and_then(|t| {
                if t.len() == 32 {
                    let mut rsp = [0u8; 32];
                    rsp.copy_from_slice(t.as_ref());
                    Ok(rsp)
                } else {
                    Err(eyre!("Invalid size"))
                }
            }) {
            Ok(t) => {
                output.push(t);
            }
            Err(err) => {
                // this is not a fatal error
                error!("Failed to retrieve key from {url}");
                error!("{err}");
            }
        }
    }
    output
}

pub fn update(cli: &cli::Cli, client: &reqwest::blocking::Client) -> Result<()> {
    let current_version = cargo_crate_version!();
    info!("Checking for updates.");
    info!("Current updater version {current_version}.");
    let current_exe = std::env::current_exe();
    let status = self_update::backends::github::Update::configure()
        .repo_owner(&cli.updater_repo_owner)
        .repo_name(&cli.updater_repo_name)
        .bin_name("dispersion")
        .show_download_progress(true)
        .no_confirm(true)
        .current_version(current_version)
        .verifying_keys(prepare_keys(&cli.updater_signature_url, client))
        .build()?
        .update()?;
    info!("Update status: `{}`!", status.version());
    // https://github.com/topgrade-rs/topgrade/blob/26f6ccf12dedd3284b5aa23d7563c7b1edc51dd7/src/self_update.rs#L59-L79
    if status.updated() {
        info!("Respawning...");

        let mut command = std::process::Command::new(current_exe?);
        command.args(std::env::args().skip(1));

        #[cfg(unix)]
        {
            let err = command.exec();
            error!("Failed to restart: {err}");
            std::process::exit(1);
        }

        #[cfg(windows)]
        {
            let status = command.status()?;
            exit(status.code().expect("This cannot return None on Windows"));
        }
    }
    Ok(())
}

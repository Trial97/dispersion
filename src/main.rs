#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use clap::Parser;
use eframe::egui;
use egui_commonmark::*;
use eyre::Result;
use log::info;

use crate::{
    application::App,
    application_states::{ApplicationStates, Release},
    install_type::InstallationType,
};

mod application;
mod application_states;
mod cli;
mod download;
mod install_type;
mod logger;
mod paths;
mod self_updater;

fn check(cli: &cli::Cli) -> Result<Release> {
    info!("check command");
    let b = self_update::backends::github::Update::configure()
        .repo_owner(&cli.repo_owner)
        .repo_name(&cli.repo_name)
        .bin_name(&cli.binary_name)
        .show_download_progress(true)
        .no_confirm(true)
        .current_version(&cli.current_version)
        .build()?;

    {
        let releases = b.get_latest_releases(&cli.current_version)?;
        let release = {
            // Filter compatible version
            let compatible_releases = releases
                .iter()
                .filter(|r| {
                    self_update::version::bump_is_compatible(&cli.current_version, &r.version)
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>();

            // Get the first version
            let release = compatible_releases.first().cloned();
            if let Some(release) = release {
                println!(
                    "v{} ({} versions compatible)",
                    release.version,
                    compatible_releases.len(),
                );

                Some(release.clone())
            } else {
                let release = releases.first();
                if let Some(release) = release {
                    println!(
                        "v{} ({} versions available)",
                        release.version,
                        releases.len(),
                    );
                    Some(release.clone())
                } else {
                    None
                }
            }
        };
        match release {
            Some(r) => {
                let i = InstallationType::new(&cli.root_dir);
                match i.find_best_asset(&r.assets, &cli.compiler) {
                    Some(best_asset) => Ok(Release {
                        name: r.name,
                        version: r.version,
                        date: r.date,
                        body: r.body,
                        asset: best_asset.clone(),
                    }),
                    None => todo!(),
                }
            }
            None => todo!(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    logger::init(&cli)?;
    let client = reqwest::blocking::Client::new();
    self_updater::update(&cli, &client)?;

    info!("check command");
    match check(&cli) {
        Ok(r) => {
            println!("New version found:");
            println!("{:#?}", r);

            let options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
                ..Default::default()
            };

            let _ = eframe::run_native(
                "My egui App",
                options,
                Box::new(|_cc| Ok(Box::new(App::new(cli, r)))),
            );
        }
        Err(_) => todo!(),
    }

    Ok(())
}

use clap::Parser;
use eyre::Result;
use log::info;

mod cli;
mod logger;
mod matcher;
mod paths;
mod self_updater;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    logger::init(&cli)?;
    let client = reqwest::blocking::Client::new();
    self_updater::update(&cli, &client)?;

    match cli.command {
        cli::Commands::Check => {
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
                            self_update::version::bump_is_compatible(
                                &cli.current_version,
                                &r.version,
                            )
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
                    Some(r) => println!("{:#?}", r),
                    None => todo!(),
                }
            }
        }
        cli::Commands::Update => {
            info!("update command")
        }
    }
    Ok(())
}

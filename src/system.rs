use regex::Regex;
use semver::Version;
use std::{
    env,
    path::{Path, PathBuf},
};

use crate::github::{PrismArtifact, PrismRelease};

fn is_arm() -> bool {
    matches!(std::env::consts::ARCH, "arm" | "aarch64")
}

#[derive(Debug, PartialEq, Clone)]
pub enum InstallationType {
    Portable,
    Appimage,
    Flatpak,
    Snap,
    Normal,
}

pub fn get_exe_root_dir() -> Option<PathBuf> {
    if let Ok(exe_path) = env::current_exe() {
        let exe_dir = exe_path.parent()?;
        Some(exe_dir.into())
    } else {
        None
    }
}

pub fn get_instalation_type(root_dir: &PathBuf) -> InstallationType {
    let portable_file_path = root_dir.join("portable.txt");
    let portable_dir_path = root_dir.join("UserData");
    match true {
        _ if Path::new("/.flatpak-info").exists() => InstallationType::Flatpak,
        _ if env::var("SNAP").is_ok() => InstallationType::Snap,
        _ if root_dir.starts_with("/tmp/mount_") => InstallationType::Appimage,
        _ if (portable_file_path.exists() && portable_file_path.is_file())
            || (portable_dir_path.exists() && portable_dir_path.is_dir()) =>
        {
            InstallationType::Portable
        }
        _ => InstallationType::Normal,
    }
}

pub fn select_valid_artifacts(
    release: &PrismRelease,
    build_artifact: String,
    installation_type: InstallationType,
) -> eyre::Result<Vec<&PrismArtifact>> {
    let artifacts: Vec<&PrismArtifact> = release
        .assets
        .iter()
        .into_iter()
        .filter(|x| !x.name.ends_with(".zsync"))
        .filter(|x| {
            !((installation_type == InstallationType::Appimage)
                ^ x.name.to_lowercase().ends_with("appimage"))
        })
        .filter(|x| {
            let asset_name = x.name.to_lowercase();
            let (platform, platform_qt_ver) = match build_artifact.to_lowercase().split_once("-qt")
            {
                Some((first, second)) => (first.to_string(), second.to_string()),
                None => (build_artifact.to_lowercase(), "6".into()),
            };
            let system_is_arm = is_arm();
            let asset_is_arm = asset_name.contains("arm64");
            let asset_is_archive = asset_name.ends_with(".zip") || asset_name.ends_with(".tar.gz");
            let for_platform = !platform.is_empty() && asset_name.contains(&platform);
            if !for_platform {
                log::info!("Rejecting {:?} because platforms do not match", asset_name);
                return false;
            }
            let for_portable = asset_name.contains("portable");
            if asset_name.contains("legacy") && !platform.contains("legacy") {
                log::info!("Rejecting {:?} because platforms do not match2", asset_name);
                return false;
            }
            if (asset_is_arm && !system_is_arm) || (!asset_is_arm && system_is_arm) {
                log::info!(
                    "Rejecting {:?} because architecture do not match",
                    asset_name
                );
                return false;
            }
            if platform.contains("windows")
                && (installation_type != InstallationType::Portable)
                && asset_is_archive
            {
                log::info!("Rejecting {:?} because it is not an installer", asset_name);
                return false;
            }
            let qt_pattern = Regex::new(r"-qt(\d+)").unwrap();
            if let Some(captures) = qt_pattern.captures(&asset_name) {
                if platform_qt_ver.is_empty()
                    || platform_qt_ver.parse::<i32>().unwrap_or(0)
                        != captures[1].parse::<i32>().unwrap_or(0)
                {
                    log::info!(
                        "Rejecting {:?} because it is not for the correct qt version {:?} vs {:?}",
                        asset_name,
                        platform_qt_ver.parse::<i32>().unwrap_or(0),
                        captures[1].parse::<i32>().unwrap_or(0)
                    );
                    return false;
                }
            }
            log::info!("{:?} vs {:?}", installation_type, for_portable);
            (installation_type == InstallationType::Portable) == for_portable
        })
        .collect();
    if artifacts.is_empty() {
        return Err(eyre::eyre!("No artifacts found"));
    }

    Ok(artifacts)
}

fn parse_semver(input: &str) -> eyre::Result<Version> {
    // Split the input by dots
    let parts: Vec<&str> = input.split('.').collect();

    // If there are 2 parts, add a third part (patch version) as "0"
    let full_semver_str = if parts.len() == 2 {
        format!("{}.{}.0", parts[0], parts[1])
    } else {
        input.to_string()
    };

    // Parse the full version string
    Ok(Version::parse(&full_semver_str)?)
}

pub fn compare_tags(v1: String, v2: String) -> eyre::Result<bool> {
    let v1 = match parse_semver(&v1) {
        Ok(version) => version,
        Err(e) => {
            log::error!("Failed to parse v1={:?}", v1);
            return Err(e.into());
        }
    };
    let v2 = match parse_semver(&v2) {
        Ok(version) => version,
        Err(e) => {
            log::error!("Failed to parse v2={:?}", v1);
            return Err(e.into());
        }
    };
    Ok(v1 > v2)
}

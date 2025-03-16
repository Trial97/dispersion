use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use glob::glob;
use log::error;

fn ensure_folder_exists(path: &Path) -> io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

fn ensure_file_exists(path: &Path) -> io::Result<()> {
    if !path.exists() {
        File::create(path)?;
    }
    Ok(())
}

fn move_file(src: &Path, dest: &Path) -> io::Result<()> {
    fs::copy(src, dest)?;
    fs::remove_file(src)?;
    Ok(())
}

pub fn backup_app_dir(
    root_path: &str,
    prism_version: &str,
    prism_git_commit: &str,
    build_artifact: &str,
) -> eyre::Result<()> {
    let manifest_path = Path::new(root_path).join("manifest.txt");
    let mut file_list: Vec<String> = Vec::new();

    // If manifest.txt exists, read the file list from it
    if manifest_path.exists() {
        let contents = fs::read_to_string(&manifest_path)?;
        file_list.extend(contents.lines().map(|line| line.trim().to_string()));
    }

    // If file_list is empty, make a guess based on the platform
    if file_list.is_empty() {
        if build_artifact.to_lowercase().contains("linux") {
            file_list.push("PrismLauncher".to_string());
            file_list.push("bin".to_string());
            file_list.push("share".to_string());
            file_list.push("lib".to_string());
        } else {
            // Windows case
            file_list.push("jars".to_string());
            file_list.push("prismlauncher.exe".to_string());
            file_list.push("prismlauncher_filelink.exe".to_string());
            file_list.push("prismlauncher_updater.exe".to_string());
            file_list.push("qtlogging.ini".to_string());
            file_list.push("imageformats".to_string());
            file_list.push("iconengines".to_string());
            file_list.push("platforms".to_string());
            file_list.push("styles".to_string());
            file_list.push("tls".to_string());
            file_list.push("qt.conf".to_string());
            file_list.push("Qt*.dll".to_string());
        }
    }

    // Create backup directory
    let app_dir = Path::new(root_path);
    let backup_dir = Path::new(root_path).join(format!(
        "backup_{}-{}",
        prism_version.replace(&['\\', '/', ':', '*', '?', '"', '<', '>', '|'][..], "_"),
        prism_git_commit
    ));

    ensure_folder_exists(&backup_dir)?;

    for glob_pattern in file_list.iter() {
        let glob_path = app_dir.join(glob_pattern);

        // Use glob crate to match files with wildcard patterns
        let files = glob(glob_path.to_str().unwrap())?;

        for entry in files {
            match entry {
                Ok(path) => {
                    let dest_path = backup_dir.join(path.strip_prefix(app_dir).unwrap());
                    ensure_folder_exists(&dest_path.parent().unwrap())?;
                    if let Err(e) = move_file(&path, &dest_path) {
                        error!(
                            "Failed to backup {} to {}: {}",
                            path.display(),
                            dest_path.display(),
                            e
                        );
                    }
                }
                Err(e) => {
                    error!("Failed to match file pattern {}: {}", glob_pattern, e);
                }
            }
        }
    }
    Ok(())
}

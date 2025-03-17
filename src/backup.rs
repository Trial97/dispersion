use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use glob::glob;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use log::error;

fn ensure_folder_exists(path: &Path) -> io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

fn move_file(src: &Path, dest: &Path) -> io::Result<()> {
    fs::copy(src, dest)?;
    fs::remove_file(src)?;
    Ok(())
}

pub fn load_manifest_files(root_path: &PathBuf, is_linux: bool) -> eyre::Result<Vec<PathBuf>> {
    let manifest_path = Path::new(root_path).join("manifest.txt");
    let mut file_list: Vec<String> = Vec::new();

    // If manifest.txt exists, read the file list from it
    if manifest_path.exists() {
        let contents = fs::read_to_string(&manifest_path)?;
        file_list.extend(contents.lines().map(|line| line.trim().to_string()));
    }
    // If file_list is empty, make a guess based on the platform
    if file_list.is_empty() {
        if is_linux {
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

    file_list.iter().try_fold(Vec::new(), |mut acc, file| {
        let pattern = root_path.join(file);
        let paths = glob(pattern.to_str().unwrap())?;
        acc.extend(paths.filter_map(Result::ok));
        Ok(acc)
    })
}

pub fn backup_app_dir(
    root_path: &PathBuf,
    prism_version: &str,
    prism_git_commit: &str,
    build_artifact: &str,
) -> eyre::Result<()> {
    // Create backup directory
    let backup_dir = Path::new(root_path).join(format!(
        "backup_{}-{}",
        prism_version.replace(&['\\', '/', ':', '*', '?', '"', '<', '>', '|'][..], "_"),
        prism_git_commit
    ));

    move_with_manifest(
        root_path,
        &backup_dir,
        build_artifact.to_lowercase().contains("linux"),
    )
}

pub fn move_with_manifest(src: &PathBuf, dst: &PathBuf, is_linux: bool) -> eyre::Result<()> {
    let file_list = load_manifest_files(src, is_linux)?;
    ensure_folder_exists(&dst)?;

    let bar = ProgressBar::new(file_list.len().try_into().unwrap());
    bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] ({pos}/{len}, ETA {eta})",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    let src = src.canonicalize()?;
    for path in file_list.iter() {
        let path = path.canonicalize()?;
        let dest_path = dst.join(path.strip_prefix(&src)?);
        ensure_folder_exists(&dest_path.parent().unwrap())?;
        if let Err(e) = move_file(&path, &dest_path) {
            error!(
                "Failed to move {} to {}: {}",
                path.display(),
                dest_path.display(),
                e
            );
        }
        bar.inc(1);
    }
    bar.finish_and_clear();
    Ok(())
}

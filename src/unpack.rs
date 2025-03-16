use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use tar::Archive;
use zip::read::ZipArchive;

fn unarchive_tar_gz(src: &Path, dest: &Path) -> eyre::Result<()> {
    let file = File::open(src)?;
    let decompressor = GzDecoder::new(file);
    let mut archive = Archive::new(decompressor);
    // Create the destination directory if it doesn't exist
    fs::create_dir_all(dest)?;

    // Extract files to the destination directory
    archive.unpack(dest)?;

    Ok(())
}

fn unarchive_zip(src: &Path, dest: &Path) -> eyre::Result<()> {
    let file = File::open(src)?;
    let mut archive = ZipArchive::new(file)?;

    // Create the destination directory if it doesn't exist
    fs::create_dir_all(dest)?;

    // Extract all files in the archive
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let path = dest.join(file.name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&path)?;
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = File::create(&path)?;
            io::copy(&mut file, &mut out_file)?;
        }
    }

    Ok(())
}

enum AchiveType {
    Zip,
    TarGz,
    None,
}

fn get_archive_type(path: &Path) -> (String, AchiveType) {
    let filename = path.file_name().unwrap().to_string_lossy();
    // First, check for `.tar.gz` suffix
    if let Some(stripped) = filename.strip_suffix(".tar.gz") {
        return (stripped.to_string(), AchiveType::TarGz);
    }

    // Then, check for `.zip` suffix
    if let Some(stripped) = filename.strip_suffix(".zip") {
        return (stripped.to_string(), AchiveType::Zip);
    }

    // Return the original path and None if no suffix matched
    ("".to_string(), AchiveType::None)
}

fn get_unique_path(base_path: &PathBuf) -> PathBuf {
    let mut unique_path = PathBuf::from(base_path);
    let mut count = 1;

    while unique_path.exists() {
        let parent = Path::new(base_path)
            .parent()
            .unwrap_or_else(|| Path::new("."));
        let stem = Path::new(base_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");

        let new_filename = format!("{}_{}", stem, count);

        unique_path = parent.join(new_filename);
        count += 1;
    }

    unique_path
}

pub fn unarchive_loop(src: &Path, dir: &Path) -> eyre::Result<PathBuf> {
    let (filename, archive_type) = get_archive_type(src);
    let new_path = dir.join(filename);
    let new_path = get_unique_path(&new_path);

    match archive_type {
        AchiveType::Zip => unarchive_zip(src, &new_path)?,
        AchiveType::TarGz => unarchive_tar_gz(src, &new_path)?,
        AchiveType::None => {
            return Err(eyre::eyre!("Unsupported file type"));
        }
    };
    fs::remove_file(src)?;
    let files: Vec<PathBuf> = fs::read_dir(&new_path)?
        .filter_map(|entry| entry.ok()) // Ignore errors
        .map(|entry| entry.path())
        .filter(|path| path.is_file()) // Keep only files
        .collect();
    // Handle cases based on file count
    match files.len() {
        1 => match unarchive_loop(&files[0], dir) {
            Ok(v) => {
                fs::remove_dir_all(new_path)?;
                Ok(v)
            }
            Err(_) => Ok(new_path),
        },
        _ => {
            return Ok(new_path);
        }
    }
}

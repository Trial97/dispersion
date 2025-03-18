use fs2::FileExt;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::PathBuf;

pub struct FileLock {
    file: std::fs::File,
    path: PathBuf,
}

impl FileLock {
    pub fn lock(path: PathBuf) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

        file.try_lock_exclusive()?; // Try to acquire an exclusive lock

        Ok(Self { file, path })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if let Err(e) = fs2::FileExt::unlock(&self.file) {
            eprintln!("Failed to unlock file: {}", e);
        }
        if let Err(e) = fs::remove_file(self.path.clone()) {
            eprintln!("Failed to remove file: {}", e);
        }
    }
}

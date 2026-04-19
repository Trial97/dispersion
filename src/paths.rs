use std::{env, path::PathBuf};

pub fn get_exe_root_dir() -> Option<PathBuf> {
    if let Ok(exe_path) = env::current_exe() {
        let exe_dir = exe_path.parent()?;
        Some(exe_dir.into())
    } else {
        None
    }
}

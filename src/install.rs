use log::error;
use std::{env, io, process::ExitStatus};
use tokio::process::Command;

async fn call_appimage_update(root_path: &str) -> eyre::Result<()> {
    // Retrieve the APPIMAGE environment variable
    if let Ok(appimage_path) = env::var("APPIMAGE") {
        let appimage_update_path = format!("{}/bin/AppImageUpdate-x86_64.AppImage", root_path);

        // Execute the AppImageUpdate with the appimage_path as an argument
        let status = Command::new(appimage_update_path)
            .arg(appimage_path)
            .spawn()
            .expect("command failed to start")
            .wait()
            .await;

        // Check the exit status of the process
        match status {
            Ok(status) if status.success() => Ok(()),
            _ => {
                error!("AppImageUpdate failed to start or finished with an error.");
                Err(eyre::eyre!("AppImageUpdate failed"))
            }
        }
    } else {
        Err(eyre::eyre!("Unsupported instalation"))
    }
}

async fn run_installer(file_path: &str) -> io::Result<ExitStatus> {
    // Create a new Command to execute the installer file
    let mut command = Command::new(file_path);

    // Set the environment variable if on Windows
    #[cfg(target_os = "windows")]
    {
        command.env("__COMPAT_LAYER", "RUNASINVOKER");
    }
    // Start the process detached
    command
        .spawn()
        .expect("command failed to start")
        .wait()
        .await
}

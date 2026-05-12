use reqwest::Client;
use reqwest::header::CONTENT_DISPOSITION;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::application_states::ProgressState;

pub async fn fetch_url(
    url: String,
    path: &Path,
    state: Arc<Mutex<ProgressState>>,
) -> eyre::Result<PathBuf> {
    let client = Client::new();
    let response = client.get(url).send().await?;
    let mut response = response.error_for_status()?;
    {
        let mut s = state.lock().unwrap();
        s.in_progress = true;
        s.progress = 0.0;
        s.status = "Starting download...".into();
    }
    let filename = response
        .headers()
        .get(CONTENT_DISPOSITION)
        .and_then(|val| val.to_str().ok())
        .and_then(|cd| cd.split("filename=").nth(1))
        .map(|name| name.trim_matches('"'))
        .map(String::from)
        .unwrap_or_else(|| "downloaded_file".to_string());
    let size = response.content_length().unwrap_or(0);

    let path_to_file = path.join(filename);
    // Open a file to write the stream to
    let mut file = File::create(&path_to_file)?;
    // Stream the response body and write it to the file chunk by chunk
    let mut downloaded: u64 = 0;
    while let Some(chunk) = response.chunk().await? {
        let current = chunk.len() as u64;
        file.write_all(&chunk)?;
        downloaded += current;
        let progress = if size > 0 {
            downloaded as f32 / size as f32
        } else {
            0.0
        };
        {
            let mut s = state.lock().unwrap();
            s.progress = progress;
            s.status = format!("Downloaded {} / {} bytes", downloaded, size);
        }
    }

    file.flush()?;
    log::info!("File downloaded successfully.");
    {
        let mut s = state.lock().unwrap();
        s.progress = 1.0;
        s.status = "Download complete".into();
        s.in_progress = false;
    }
    Ok(path_to_file)
}

use reqwest::Client;
use reqwest::header::CONTENT_DISPOSITION;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub async fn fetch_url(url: url::Url, path: &Path) -> eyre::Result<PathBuf> {
    let client = Client::new();
    let response = client.get(url).send().await?;
    let mut response = response.error_for_status()?;
    let filename = response
        .headers()
        .get(CONTENT_DISPOSITION)
        .and_then(|val| val.to_str().ok())
        .and_then(|cd| cd.split("filename=").nth(1))
        .map(|name| name.trim_matches('"'))
        .map(String::from)
        .unwrap_or_else(|| "downloaded_file".to_string());

    let path_to_file = path.join(filename);
    // Open a file to write the stream to
    let mut file = File::create(&path_to_file)?;
    // Stream the response body and write it to the file chunk by chunk
    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk)?;
    }

    file.flush()?;
    log::info!("File downloaded successfully.");
    Ok(path_to_file)
}

use std::{
    path::Path,
    process::exit,
    sync::{Arc, Mutex},
    thread,
};

use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::{
    application_states::{self, ApplicationStates, ProgressState},
    cli,
    download::fetch_url,
};

pub struct App {
    cli: cli::Cli,
    page: Arc<Mutex<ApplicationStates>>,
    cache: CommonMarkCache,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, frame: &mut eframe::Frame) {
        let mut page = self.page.lock().unwrap();
        match page.clone() {
            ApplicationStates::DoNothing => exit(0),
            ApplicationStates::SkipUpdate => exit(1),
            ApplicationStates::Ask(release) => {
                let Some(ref markdown) = release.body else {
                    return todo!();
                };
                ui.heading(&release.name);
                ui.horizontal_centered(|ui| {
                    CommonMarkViewer::new().show(ui, &mut self.cache, markdown.as_str());
                });
                ui.horizontal_centered(|ui| {
                    if ui.button("Close").clicked() {
                        *page = ApplicationStates::DoNothing;
                    }
                    if ui.button("Skip").clicked() {
                        *page = ApplicationStates::SkipUpdate;
                    }
                    if ui.button("Update").clicked() {
                        let mutex = Arc::new(Mutex::new(application_states::ProgressState {
                            progress: 0.0,
                            status: "Starting".to_string(),
                            in_progress: true,
                        }));
                        let download_url = release.asset.download_url.clone();
                        let mutex_clone = mutex.clone();
                        tokio::spawn(async move {
                            // self.downloadAsset(&release.asset.download_url, mutex);
                            let artifact_path =
                                match fetch_url(download_url, Path::new("."), mutex).await {
                                    Ok(v) => v,
                                    Err(err) => {
                                        log::error!("Failed to download artifact: {:?}", err);
                                        return Err(err);
                                    }
                                };
                            Ok(())
                        });
                        *page = ApplicationStates::Progress(mutex_clone);
                    }
                });
            }
            ApplicationStates::Progress(mutex) => todo!(),
            ApplicationStates::Download(release_asset) => todo!(),
            ApplicationStates::RemoveOld => todo!(),
            ApplicationStates::Extract(_) => todo!(),
            ApplicationStates::Execute(_) => todo!(),
            ApplicationStates::Backup => todo!(),
            ApplicationStates::Error(_) => todo!(),
        }
    }
}

impl App {
    pub fn new(cli: cli::Cli, release: application_states::Release) -> App {
        App {
            page: Arc::new(Mutex::new(ApplicationStates::Ask(release))),
            cli,
            cache: CommonMarkCache::default(),
        }
    }
    pub fn downloadAsset(&mut self, url: &str, state: Arc<Mutex<ProgressState>>) {}
}

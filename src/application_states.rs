use std::sync::{Arc, Mutex};

use self_update::update::ReleaseAsset;

#[derive(Clone, Debug, Default)]
pub struct Release {
    pub name: String,
    pub version: String,
    pub date: String,
    pub body: Option<String>,
    pub asset: ReleaseAsset,
}

#[derive(Default)]
pub struct ProgressState {
    pub progress: f32,
    pub status: String,
    pub in_progress: bool,
}

#[derive(Clone)]
pub enum ApplicationStates {
    DoNothing,
    SkipUpdate,
    Ask(Release),
    Progress(Arc<Mutex<ProgressState>>),
    Download(ReleaseAsset),
    RemoveOld,
    Extract(String),
    Execute(String),
    Backup,
    Error(String),
}

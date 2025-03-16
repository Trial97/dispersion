use chrono::{DateTime, Utc};
use clap::ValueEnum;
use octocrab::Octocrab;
use url::Url;

use crate::cli::CommandArgs;

#[derive(Debug, Clone, ValueEnum)]
pub enum ReleaseType {
    Stable,
    Nightly,
}

#[derive(Debug)]
pub struct PrismRelease {
    pub name: String,
    pub tag: String,
    pub created_at: DateTime<Utc>,
    pub assets: Vec<PrismArtifact>,
    pub body: Option<String>,
}

#[derive(Debug)]
pub struct PrismArtifact {
    pub node_id: String,
    pub name: String,
    pub size_in_bytes: usize,
    pub url: Url,
    pub download_url: Url,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

async fn get_latest_release(octocrab: &Octocrab, cfg: &CommandArgs) -> eyre::Result<PrismRelease> {
    let release = octocrab
        .repos(&cfg.repo_owner, &cfg.repo_name)
        .releases()
        .get_latest()
        .await?;
    let artifacts = release
        .assets
        .iter()
        .map(|asset| PrismArtifact {
            node_id: asset.node_id.clone(),
            name: asset.name.clone(),
            size_in_bytes: asset.size.try_into().unwrap(),
            url: asset.url.clone(),
            download_url: asset.browser_download_url.clone(),
            created_at: asset.created_at,
            updated_at: asset.updated_at,
        })
        .collect();
    Ok(PrismRelease {
        name: release.name.unwrap_or("".to_string()),
        tag: release.tag_name.clone(),
        created_at: release.created_at.unwrap(),
        assets: artifacts,
        body: release.body,
    })
}

async fn get_commit_messages(
    octocrab: &Octocrab,
    cfg: &CommandArgs,
    base_sha: &str,
    head_sha: &str,
) -> octocrab::Result<String> {
    let comparison = octocrab
        .commits(&cfg.repo_owner, &cfg.repo_name)
        .compare(base_sha, head_sha)
        .send()
        .await?;

    let changelog = comparison
        .commits
        .iter()
        .map(|commit| {
            let sha = &commit.sha;
            let message = &commit.commit.message;
            let url = &commit.html_url;
            format!("[{sha}] {message} ({url})")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let full_changelog_link = format!("\nFull changelog: {}", comparison.html_url);

    Ok(format!("#Changelog\n\n{changelog}\n{full_changelog_link}").to_owned())
}

async fn get_latest_workflow_run(
    octocrab: &Octocrab,
    cfg: &CommandArgs,
) -> eyre::Result<PrismRelease> {
    let runs = octocrab
        .workflows(&cfg.repo_owner, &cfg.repo_name)
        .list_runs(&cfg.workflow_name)
        .branch(&cfg.branch)
        .per_page(1)
        .send()
        .await?;
    let latest_run = runs.items.first().unwrap();
    let page = octocrab
        .actions()
        .list_workflow_run_artifacts(&cfg.repo_owner, &cfg.repo_name, latest_run.id)
        .send()
        .await?;
    let artifacts = page
        .value
        .unwrap()
        .items
        .iter()
        .map(|asset| PrismArtifact {
            node_id: asset.node_id.clone(),
            name: asset.name.clone(),
            size_in_bytes: asset.size_in_bytes,
            url: asset.url.clone(),
            download_url: format!(
                "https://nightly.link/{}/{}/actions/artifacts/{}.zip",
                cfg.repo_owner,
                cfg.repo_name,
                asset.id.to_string()
            )
            .parse()
            .unwrap(),
            created_at: asset.created_at,
            updated_at: asset.updated_at,
        })
        .collect();

    let changelog = get_commit_messages(
        octocrab,
        cfg,
        cfg.git_commit.clone().unwrap().as_str(),
        &latest_run.head_sha,
    )
    .await?;
    Ok(PrismRelease {
        name: latest_run.name.clone(),
        tag: latest_run.head_sha.clone(),
        created_at: latest_run.created_at,
        assets: artifacts,
        body: Some(changelog),
    })
}

pub async fn get_latest(cfg: &CommandArgs) -> eyre::Result<PrismRelease> {
    let octocrab = Octocrab::builder().build()?;
    match cfg.release_type {
        ReleaseType::Stable => get_latest_release(&octocrab, cfg).await,
        ReleaseType::Nightly => get_latest_workflow_run(&octocrab, cfg).await,
    }
}

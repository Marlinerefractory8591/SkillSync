use crate::services::github::GitHubService;
use serde::Serialize;

const RELEASE_REPOSITORY: &str = "https://github.com/tomaszboloz/SkillSync";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateInfo {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub release_url: String,
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub async fn check_app_update() -> Result<AppUpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let release_url = format!("{}/releases", RELEASE_REPOSITORY);
    let release = GitHubService::check_latest_version(RELEASE_REPOSITORY).await?;

    let latest_version =
        release.map(|release| release.tag_name.trim_start_matches(['v', 'V']).to_string());
    let update_available = latest_version
        .as_deref()
        .map(|latest| GitHubService::is_newer_version(latest, &current_version))
        .unwrap_or(false);

    Ok(AppUpdateInfo {
        current_version,
        latest_version,
        update_available,
        release_url,
    })
}

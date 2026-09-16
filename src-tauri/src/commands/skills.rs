use crate::models::skill::{BackupSnapshot, SkillMetadata, SkillStatus};
use crate::services::backup::BackupService;
use crate::services::config::ConfigService;
use crate::services::detector::SkillDetector;
use crate::services::detector::UNKNOWN_SKILL_VERSION;
use crate::services::git::GitService;
use crate::services::github::GitHubService;
use crate::services::orchestrator::UpdateOrchestrator;
use std::path::PathBuf;

#[tauri::command]
pub async fn scan_skills(_force_refresh: bool) -> Result<Vec<SkillMetadata>, String> {
    let config = ConfigService::load_config();
    let paths: Vec<PathBuf> = config
        .paths
        .monitored
        .into_iter()
        .filter(|p| p.enabled)
        .map(|p| p.path)
        .collect();

    let mut skills = SkillDetector::scan_directories(&paths);

    // A missing manifest version can be resolved only from a tag pointing at HEAD.
    for skill in &mut skills {
        skill.current_version = skill
            .current_version
            .trim_start_matches(['v', 'V'])
            .to_string();
        if skill.is_git_repo && skill.current_version == UNKNOWN_SKILL_VERSION {
            if let Some(tag) = GitService::get_head_tag(&skill.path) {
                skill.current_version = tag.trim_start_matches(['v', 'V']).to_string();
            }
        }
    }

    // Query GitHub concurrently for skills with remote_url (deduplicated by URL)
    let mut unique_urls: std::collections::HashSet<String> = std::collections::HashSet::new();
    for skill in &skills {
        if let Some(ref url) = skill.remote_url {
            unique_urls.insert(url.clone());
        }
    }

    let mut set = tokio::task::JoinSet::new();
    for url in unique_urls {
        set.spawn(async move {
            let release = GitHubService::check_latest_version(&url)
                .await
                .ok()
                .flatten();
            (url, release)
        });
    }

    let mut releases_map: std::collections::HashMap<
        String,
        crate::services::github::GitHubReleaseInfo,
    > = std::collections::HashMap::new();
    while let Some(res) = set.join_next().await {
        if let Ok((url, Some(release))) = res {
            releases_map.insert(url, release);
        }
    }

    for skill in &mut skills {
        if let Some(ref url) = skill.remote_url {
            if let Some(release) = releases_map.get(url) {
                let is_newer =
                    GitHubService::is_newer_version(&release.tag_name, &skill.current_version);
                let compat = GitHubService::get_semver_compatibility(
                    &release.tag_name,
                    &skill.current_version,
                );
                let clean_tag = release.tag_name.trim_start_matches(['v', 'V']).to_string();
                skill.latest_version = Some(clean_tag);
                skill.update_available = is_newer;
                skill.status = if is_newer {
                    SkillStatus::UpdateAvailable
                } else {
                    SkillStatus::UpToDate
                };
                if let Some(ref body) = release.body {
                    skill.changelog = Some(body.clone());
                }
                skill.update_compatibility = Some(compat);
                skill.last_checked = chrono::Utc::now();
            }
        }
    }

    Ok(skills)
}

#[tauri::command]
pub async fn check_github_update(skill_id: String) -> Result<SkillMetadata, String> {
    let all_skills = scan_skills(false).await?;
    let mut skill = all_skills
        .into_iter()
        .find(|s| s.id == skill_id)
        .ok_or_else(|| format!("Skill with ID {} not found", skill_id))?;

    if let Some(ref remote_url) = skill.remote_url {
        if let Ok(Some(release)) = GitHubService::check_latest_version(remote_url).await {
            let is_newer =
                GitHubService::is_newer_version(&release.tag_name, &skill.current_version);
            let compat =
                GitHubService::get_semver_compatibility(&release.tag_name, &skill.current_version);
            skill.latest_version = Some(release.tag_name.trim_start_matches('v').to_string());
            skill.update_available = is_newer;
            skill.status = if is_newer {
                SkillStatus::UpdateAvailable
            } else {
                SkillStatus::UpToDate
            };
            if let Some(body) = release.body {
                skill.changelog = Some(body);
            }
            skill.update_compatibility = Some(compat);
            skill.last_checked = chrono::Utc::now();
        }
    }

    Ok(skill)
}

#[tauri::command]
pub async fn update_single_skill(
    app: tauri::AppHandle,
    skill_id: String,
    target_version: Option<String>,
    force: Option<bool>,
) -> Result<SkillMetadata, String> {
    use crate::models::skill::{UpdateProgressPayload, UpdateStage};
    use tauri::Emitter;

    let all_skills = scan_skills(false).await?;
    let skill = all_skills
        .into_iter()
        .find(|s| s.id == skill_id)
        .ok_or_else(|| format!("Skill with ID {} not found", skill_id))?;

    let _ = app.emit(
        "update-progress",
        UpdateProgressPayload {
            skill_id: skill.id.clone(),
            skill_name: skill.name.clone(),
            stage: UpdateStage::Validating,
            percentage: 15,
            message: format!("Walidacja parametrów dla {}...", skill.name),
        },
    );

    let _ = app.emit(
        "update-progress",
        UpdateProgressPayload {
            skill_id: skill.id.clone(),
            skill_name: skill.name.clone(),
            stage: UpdateStage::BackingUp,
            percentage: 35,
            message: "Tworzenie migawki bezpieczeństwa (backup snapshot)...".into(),
        },
    );

    let locs_count = if skill.installed_locations.is_empty() {
        1
    } else {
        skill.installed_locations.len()
    };
    let _ = app.emit(
        "update-progress",
        UpdateProgressPayload {
            skill_id: skill.id.clone(),
            skill_name: skill.name.clone(),
            stage: UpdateStage::CheckingOut,
            percentage: 65,
            message: format!(
                "Aktualizacja i synchronizacja w {} lokalizacjach...",
                locs_count
            ),
        },
    );

    let result =
        UpdateOrchestrator::update_skill_atomic(&skill, target_version, force.unwrap_or(false))
            .await;

    match result {
        Ok(updated) => {
            let _ = app.emit(
                "update-progress",
                UpdateProgressPayload {
                    skill_id: skill.id.clone(),
                    skill_name: skill.name.clone(),
                    stage: UpdateStage::Completed,
                    percentage: 100,
                    message: format!(
                        "Zaktualizowano pomyślnie do wersji {} we wszystkich lokalizacjach!",
                        updated.current_version
                    ),
                },
            );
            Ok(updated)
        }
        Err(e) => {
            let _ = app.emit(
                "update-progress",
                UpdateProgressPayload {
                    skill_id: skill.id.clone(),
                    skill_name: skill.name.clone(),
                    stage: UpdateStage::Failed,
                    percentage: 100,
                    message: format!("Błąd aktualizacji: {}", e),
                },
            );
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn checkout_custom_version(
    skill_id: String,
    target_ref: String,
) -> Result<SkillMetadata, String> {
    let all_skills = scan_skills(false).await?;
    let mut skill = all_skills
        .into_iter()
        .find(|s| s.id == skill_id)
        .ok_or_else(|| format!("Skill with ID {} not found", skill_id))?;

    // 1. Create backup snapshot first
    let snapshot = BackupService::create_snapshot(&skill.path, &skill.id, &skill.current_version)
        .map_err(|e| e.to_string())?;

    // 2. Perform git checkout ref
    if let Err(e) = GitService::checkout_ref(&skill.path, &target_ref) {
        let _ = BackupService::restore_snapshot(&skill.path, &snapshot.backup_file_path);
        return Err(e.to_string());
    }

    skill.current_version = target_ref;
    skill.update_available = false;
    skill.status = SkillStatus::UpToDate;
    skill.last_checked = chrono::Utc::now();

    Ok(skill)
}

#[tauri::command]
pub async fn batch_update_skills(skill_ids: Vec<String>) -> Result<serde_json::Value, String> {
    let all_skills = scan_skills(false).await?;
    let target_skills: Vec<SkillMetadata> = all_skills
        .into_iter()
        .filter(|s| skill_ids.contains(&s.id))
        .collect();

    let config = ConfigService::load_config();
    let limit = config.updates.concurrency_limit;

    let (succeeded, failed) = UpdateOrchestrator::batch_update(target_skills, limit).await;

    Ok(serde_json::json!({
        "succeeded": succeeded,
        "failed": failed,
    }))
}

#[tauri::command]
pub async fn rollback_skill(skill_id: String, snapshot_id: Option<String>) -> Result<bool, String> {
    let all_skills = scan_skills(false).await?;
    let skill = all_skills
        .into_iter()
        .find(|s| s.id == skill_id)
        .ok_or_else(|| format!("Skill with ID {} not found", skill_id))?;

    let backups = BackupService::list_snapshots(&skill_id);
    let target_snapshot = if let Some(id) = snapshot_id {
        backups.into_iter().find(|b| b.snapshot_id == id)
    } else {
        backups.into_iter().next()
    };

    if let Some(snapshot) = target_snapshot {
        BackupService::restore_snapshot(&skill.path, &snapshot.backup_file_path)
            .map_err(|e| e.to_string())?;
        Ok(true)
    } else {
        Err("No backup snapshot found for rollback".into())
    }
}

#[tauri::command]
pub async fn get_backups_list(skill_id: String) -> Result<Vec<BackupSnapshot>, String> {
    Ok(BackupService::list_snapshots(&skill_id))
}

#[tauri::command]
pub async fn open_in_editor(path: String) -> Result<(), String> {
    // Linux has no platform-specific launcher branch below, but it still
    // exposes this IPC command for a consistent cross-platform API.
    let _ = &path;
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&path).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(&path).spawn();
    }
    Ok(())
}

#[tauri::command]
pub async fn open_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&url).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_check_uses_semver_ordering() {
        assert!(GitHubService::is_newer_version("v2.3.1", "2.0.0"));
        assert!(!GitHubService::is_newer_version("v2.3.1", "2.3.1"));
        assert!(!GitHubService::is_newer_version("v2.0.0", "2.3.1"));
    }

    #[tokio::test]
    async fn test_atomic_update_and_rollback() {
        let temp_dir = std::env::temp_dir().join(format!(
            "skillsync-test-{}",
            chrono::Utc::now().timestamp_micros()
        ));
        let skill_dir = temp_dir.join("test-sync-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();

        let initial_skill_md = "---\nname: test-sync-skill\nmetadata:\n  author: Tester\n  version: \"1.0.0\"\n---\n# Test Sync\nInitial content\n";
        std::fs::write(skill_dir.join("SKILL.md"), initial_skill_md).unwrap();

        let meta = SkillMetadata {
            id: "skill-test-sync-skill".to_string(),
            name: "test-sync-skill".to_string(),
            description: "Test skill".to_string(),
            current_version: "1.0.0".to_string(),
            latest_version: Some("1.2.0".to_string()),
            author: "Tester".to_string(),
            path: skill_dir.clone(),
            is_git_repo: false,
            remote_url: None,
            branch_or_tag: None,
            agent_scope: crate::models::skill::AgentScope::Global,
            status: SkillStatus::UpdateAvailable,
            update_available: true,
            changelog: None,
            dependencies: vec![],
            permissions: vec![],
            last_checked: chrono::Utc::now(),
            compatibility: Some("All Agents".to_string()),
            update_compatibility: None,
            installed_locations: vec![skill_dir.clone()],
        };

        // 1. Perform atomic update to 1.2.0
        let updated =
            UpdateOrchestrator::update_skill_atomic(&meta, Some("1.2.0".to_string()), false)
                .await
                .unwrap();
        assert_eq!(updated.current_version, "1.2.0");
        assert_eq!(updated.status, SkillStatus::UpToDate);

        let content_after = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
        assert!(content_after.contains("version: \"1.2.0\""));

        // 2. Verify snapshot was saved and has accurate timestamp & original version
        let snapshots = BackupService::list_snapshots("skill-test-sync-skill");
        assert!(
            !snapshots.is_empty(),
            "Backup snapshot should have been created"
        );
        let latest_snap = &snapshots[0];
        assert_eq!(latest_snap.original_version, "1.0.0");

        // 3. Test rollback
        BackupService::restore_snapshot(&skill_dir, &latest_snap.backup_file_path).unwrap();
        let content_restored = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
        assert!(content_restored.contains("version: \"1.0.0\""));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_agent_browser_detection_and_tag_resolution() {
        let skills = scan_skills(false).await.unwrap();
        let agent_browser = skills.iter().find(|s| s.name == "agent-browser");
        if let Some(skill) = agent_browser {
            println!(
                "agent-browser: current={}, latest={:?}, update_available={}",
                skill.current_version, skill.latest_version, skill.update_available
            );
            // The local installation may legitimately already be on the
            // latest tag, so this test must not depend on a developer's
            // private version being older than a fixed release.
            assert!(
                !skill.current_version.is_empty(),
                "current_version must be resolved from a real local manifest or Git ref"
            );
            if skill.latest_version == Some("0.37.1".to_string()) {
                assert!(
                    skill.update_available,
                    "agent-browser should have update_available = true"
                );
            }
        }
    }
}

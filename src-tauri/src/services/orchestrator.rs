use crate::errors::SkillSyncError;
use crate::models::skill::{SkillMetadata, SkillStatus};
use crate::services::backup::BackupService;
use crate::services::git::GitService;
use crate::services::github::GitHubService;
use crate::services::manifest::SkillManifest;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct UpdateOrchestrator;

impl UpdateOrchestrator {
    pub async fn update_skill_atomic(
        skill: &SkillMetadata,
        target_version: Option<String>,
        allow_dirty_worktree: bool,
    ) -> Result<SkillMetadata, SkillSyncError> {
        let target_tag = target_version.unwrap_or_else(|| {
            skill
                .latest_version
                .clone()
                .unwrap_or_else(|| skill.current_version.clone())
        });

        let mut locations: Vec<PathBuf> = skill.installed_locations.clone();
        if !locations.contains(&skill.path) {
            locations.push(skill.path.clone());
        }

        // Deduplicate canonical target paths
        let mut canonical_targets: Vec<PathBuf> = Vec::new();
        for loc in &locations {
            let canon = fs::canonicalize(loc).unwrap_or_else(|_| loc.clone());
            if !canonical_targets.contains(&canon) && canon.exists() {
                canonical_targets.push(canon);
            }
        }

        if canonical_targets.is_empty() && !skill.path.exists() {
            return Err(SkillSyncError::FileSystem(
                "Directory does not exist".into(),
            ));
        }

        // Never snapshot, check out, or rewrite an arbitrary directory. The
        // detector should make this guard unreachable in normal use, but it is
        // the transaction-level safety net for stale UI state and custom IPC.
        for target in &canonical_targets {
            SkillManifest::validate(target).map_err(|reason| {
                SkillSyncError::InvalidManifest(format!("{}: {reason}", target.display()))
            })?;
        }

        // 1. Stage: Create Atomic Snapshots for all target paths
        let mut snapshots = Vec::new();
        for target in &canonical_targets {
            if let Ok(snap) =
                BackupService::create_snapshot(target, &skill.id, &skill.current_version)
            {
                snapshots.push((target.clone(), snap));
            }
        }

        // 2. Stage: Perform updates across all locations
        for target in &canonical_targets {
            // A. If Git repo, perform fetch and checkout
            if GitService::is_git_repository(target) {
                if let Err(e) =
                    GitService::fetch_and_checkout_tag(target, &target_tag, allow_dirty_worktree)
                {
                    for (t, snap) in &snapshots {
                        let _ = BackupService::restore_snapshot(t, &snap.backup_file_path);
                    }
                    return Err(e);
                }
            } else if let Some(ref remote_url) = skill.remote_url {
                // If not git repo, fetch latest upstream SKILL.md if available
                if let Some(upstream_content) =
                    GitHubService::fetch_raw_skill_md(remote_url, &target_tag, &skill.name).await
                {
                    let skill_md_path = target.join("SKILL.md");
                    let _ = fs::write(&skill_md_path, upstream_content);
                }
            }

            // B. Update manifests on disk (SKILL.md, skill.json, package.json)
            if let Err(e) = Self::update_skill_md_version(target, &target_tag) {
                for (t, snap) in &snapshots {
                    let _ = BackupService::restore_snapshot(t, &snap.backup_file_path);
                }
                return Err(e);
            }

            Self::update_skill_json_version(target, &target_tag)?;
            if SkillManifest::is_explicit_package_skill(target) {
                Self::update_package_json_version(target, &target_tag)?;
            }

            // C. Stage: Post-Update Integrity Verification
            if !Self::verify_integrity(target) {
                for (t, snap) in &snapshots {
                    let _ = BackupService::restore_snapshot(t, &snap.backup_file_path);
                }
                return Err(SkillSyncError::IntegrityCheckFailed(format!(
                    "Skill manifest is corrupted or missing after update in {:?}",
                    target
                )));
            }
        }

        // Return updated metadata
        let mut updated = skill.clone();
        updated.current_version = target_tag.clone();
        updated.latest_version = Some(target_tag);
        updated.update_available = false;
        updated.status = SkillStatus::UpToDate;
        updated.update_compatibility = None;
        updated.last_checked = chrono::Utc::now();

        Ok(updated)
    }

    pub fn update_skill_md_version(dir: &Path, new_version: &str) -> Result<(), SkillSyncError> {
        let skill_md_path = dir.join("SKILL.md");
        if !skill_md_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&skill_md_path)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut updated = false;

        if lines.first().map(|l| l.trim()) == Some("---") {
            let mut end_idx = None;
            for (i, line) in lines.iter().enumerate().skip(1) {
                if line.trim() == "---" {
                    end_idx = Some(i);
                    break;
                }
            }

            if let Some(end) = end_idx {
                for line in lines.iter_mut().take(end).skip(1) {
                    let trimmed = line.trim_start();
                    if trimmed.starts_with("version:") {
                        let indent_len = line.len() - trimmed.len();
                        let indent = &line[..indent_len];
                        *line = format!("{}version: \"{}\"", indent, new_version);
                        updated = true;
                        break;
                    }
                }
                if !updated {
                    // Check if metadata: section exists
                    let mut meta_idx = None;
                    for (i, line) in lines.iter().enumerate().take(end).skip(1) {
                        if line.trim() == "metadata:" {
                            meta_idx = Some(i);
                            break;
                        }
                    }
                    if let Some(mi) = meta_idx {
                        lines.insert(mi + 1, format!("  version: \"{}\"", new_version));
                    } else {
                        lines.insert(1, format!("version: \"{}\"", new_version));
                    }
                    updated = true;
                }
            }
        }

        if !updated {
            let frontmatter = format!("---\nversion: \"{}\"\n---\n\n", new_version);
            let new_content = format!("{}{}", frontmatter, content);
            fs::write(&skill_md_path, new_content)?;
        } else {
            let mut new_content = lines.join("\n");
            if content.ends_with('\n') {
                new_content.push('\n');
            }
            fs::write(&skill_md_path, new_content)?;
        }

        Ok(())
    }

    pub fn update_skill_json_version(dir: &Path, new_version: &str) -> Result<(), SkillSyncError> {
        let skill_json_path = dir.join("skill.json");
        if !skill_json_path.exists() {
            return Ok(());
        }
        if let Ok(content) = fs::read_to_string(&skill_json_path) {
            if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(obj) = val.as_object_mut() {
                    obj.insert(
                        "version".to_string(),
                        serde_json::Value::String(new_version.to_string()),
                    );
                    if let Ok(serialized) = serde_json::to_string_pretty(&val) {
                        let _ = fs::write(&skill_json_path, serialized);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn update_package_json_version(
        dir: &Path,
        new_version: &str,
    ) -> Result<(), SkillSyncError> {
        if !SkillManifest::is_explicit_package_skill(dir) {
            return Ok(());
        }

        let pkg_json_path = dir.join("package.json");
        if !pkg_json_path.exists() {
            return Ok(());
        }
        if let Ok(content) = fs::read_to_string(&pkg_json_path) {
            if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(obj) = val.as_object_mut() {
                    obj.insert(
                        "version".to_string(),
                        serde_json::Value::String(new_version.to_string()),
                    );
                    if let Ok(serialized) = serde_json::to_string_pretty(&val) {
                        let _ = fs::write(&pkg_json_path, serialized);
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn batch_update(
        skills: Vec<SkillMetadata>,
        concurrency_limit: usize,
    ) -> (usize, usize) {
        let semaphore = Arc::new(Semaphore::new(concurrency_limit));
        let mut handles = Vec::new();

        for skill in skills {
            let sem = semaphore.clone();
            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire_owned().await.unwrap();
                Self::update_skill_atomic(&skill, None, false).await
            }));
        }

        let mut succeeded = 0;
        let mut failed = 0;

        for handle in handles {
            match handle.await {
                Ok(Ok(_)) => succeeded += 1,
                _ => failed += 1,
            }
        }

        (succeeded, failed)
    }

    fn verify_integrity(path: &Path) -> bool {
        SkillManifest::validate(path).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::skill::AgentScope;

    fn fixture_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "skillsync-orchestrator-{name}-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ))
    }

    fn metadata_for(path: PathBuf) -> SkillMetadata {
        SkillMetadata {
            id: "skill-fixture".to_string(),
            name: "fixture".to_string(),
            description: "Fixture skill".to_string(),
            current_version: "1.0.0".to_string(),
            latest_version: Some("1.1.0".to_string()),
            author: "Test".to_string(),
            path: path.clone(),
            is_git_repo: false,
            remote_url: None,
            branch_or_tag: None,
            agent_scope: AgentScope::Global,
            status: SkillStatus::UpdateAvailable,
            update_available: true,
            changelog: None,
            dependencies: vec![],
            permissions: vec![],
            last_checked: chrono::Utc::now(),
            compatibility: None,
            update_compatibility: None,
            installed_locations: vec![path],
        }
    }

    #[tokio::test]
    async fn rejects_a_directory_without_a_skill_manifest_before_mutation() {
        let dir = fixture_dir("missing-manifest");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("package.json"), r#"{"name":"documentation"}"#).unwrap();
        let metadata = metadata_for(dir.clone());

        let error = UpdateOrchestrator::update_skill_atomic(&metadata, None, false)
            .await
            .expect_err("ordinary package directories must never be updated");
        assert!(matches!(error, SkillSyncError::InvalidManifest(_)));
        assert_eq!(
            fs::read_to_string(dir.join("package.json")).unwrap(),
            r#"{"name":"documentation"}"#
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn ordinary_package_version_is_never_rewritten() {
        let dir = fixture_dir("ordinary-package");
        fs::create_dir_all(&dir).unwrap();
        let original = r#"{"name":"documentation","version":"1.0.0"}"#;
        fs::write(dir.join("package.json"), original).unwrap();

        UpdateOrchestrator::update_package_json_version(&dir, "2.0.0").unwrap();
        assert_eq!(
            fs::read_to_string(dir.join("package.json")).unwrap(),
            original
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn explicit_package_skill_version_is_rewritten() {
        let dir = fixture_dir("package-skill");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("package.json"),
            r#"{"name":"fixture","version":"1.0.0","skill":true}"#,
        )
        .unwrap();

        UpdateOrchestrator::update_package_json_version(&dir, "2.0.0").unwrap();
        let package: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.join("package.json")).unwrap()).unwrap();
        assert_eq!(package["version"], "2.0.0");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn integrity_check_rejects_malformed_skill_json() {
        let dir = fixture_dir("corrupted-json");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("skill.json"), "{invalid").unwrap();

        assert!(!UpdateOrchestrator::verify_integrity(&dir));
        let _ = fs::remove_dir_all(dir);
    }
}

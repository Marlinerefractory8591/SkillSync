use crate::commands::skills::apply_github_release;
use crate::models::skill::{SkillMetadata, SkillStatus};
use crate::services::git::GitService;
use crate::services::github::GitHubService;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

const REQUEST_INTERVAL: std::time::Duration = std::time::Duration::from_millis(350);

struct ScanJob {
    generation: u64,
    app: AppHandle,
    skill: SkillMetadata,
}

/// A single background worker deliberately spaces remote checks. Discovery is
/// quick and local; upstream verification is serial so a large install never
/// creates a GitHub/Git connection burst.
pub struct ScanQueue {
    sender: UnboundedSender<ScanJob>,
    generation: Arc<AtomicU64>,
}

impl ScanQueue {
    pub fn new() -> Self {
        let (sender, mut receiver) = unbounded_channel::<ScanJob>();
        let generation = Arc::new(AtomicU64::new(0));
        let worker_generation = Arc::clone(&generation);

        tauri::async_runtime::spawn(async move {
            while let Some(job) = receiver.recv().await {
                if worker_generation.load(Ordering::Acquire) != job.generation {
                    continue;
                }

                let skill = refresh_upstream(job.skill).await;
                if worker_generation.load(Ordering::Acquire) == job.generation {
                    let _ = job.app.emit("scan-result", skill);
                }
                tokio::time::sleep(REQUEST_INTERVAL).await;
            }
        });

        Self { sender, generation }
    }

    pub fn enqueue(&self, app: AppHandle, skills: impl IntoIterator<Item = SkillMetadata>) {
        let generation = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
        for skill in skills {
            if skill.remote_url.is_some() {
                let _ = self.sender.send(ScanJob {
                    generation,
                    app: app.clone(),
                    skill,
                });
            }
        }
    }
}

impl Default for ScanQueue {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) async fn refresh_upstream(mut skill: SkillMetadata) -> SkillMetadata {
    // A manually selected branch is an explicit opt-out from release tags.
    // Automatically detected branches must never have that effect: a
    // repository publishing SemVer tags is tracked by its newest tag.
    if skill.is_git_repo {
        if let Some(branch) = skill
            .branch_override
            .clone()
            .filter(|branch| !branch.trim().is_empty())
        {
            return refresh_branch_upstream(skill, branch).await;
        }
    }

    if skill.is_git_repo {
        let path = skill.path.clone();
        match tauri::async_runtime::spawn_blocking(move || GitService::get_latest_remote_tag(&path))
            .await
        {
            Ok(Ok(Some(tag))) => {
                let clean_tag = tag.trim_start_matches(['v', 'V']).to_string();
                skill.branch_or_tag = Some(tag.clone());
                skill.latest_version = Some(clean_tag.clone());
                skill.update_available =
                    GitHubService::is_newer_version(&clean_tag, &skill.current_version);
                skill.status = if skill.update_available {
                    SkillStatus::UpdateAvailable
                } else {
                    SkillStatus::UpToDate
                };
                skill.update_compatibility = Some(GitHubService::get_semver_compatibility(
                    &clean_tag,
                    &skill.current_version,
                ));
                skill.last_checked = chrono::Utc::now();
                return skill;
            }
            Ok(Ok(None)) => {
                let path = skill.path.clone();
                let detected_branch = skill.detected_branch.clone();
                match tauri::async_runtime::spawn_blocking(move || {
                    GitService::resolve_fallback_branch(&path, detected_branch.as_deref())
                })
                .await
                {
                    Ok(Ok(branch)) => return refresh_branch_upstream(skill, branch).await,
                    Ok(Err(error)) => {
                        skill.status = SkillStatus::Error(format!(
                            "Repozytorium nie ma tagu SemVer ani dostępnej gałęzi do śledzenia: {}",
                            error
                        ));
                    }
                    Err(error) => {
                        skill.status = SkillStatus::Error(format!(
                            "Kolejka sprawdzania gałęzi została przerwana: {}",
                            error
                        ));
                    }
                }
                skill.update_available = false;
                skill.last_checked = chrono::Utc::now();
                return skill;
            }
            Ok(Err(error)) => {
                skill.status =
                    SkillStatus::Error(format!("Nie udało się sprawdzić tagów Git: {}", error));
                skill.update_available = false;
                skill.last_checked = chrono::Utc::now();
                return skill;
            }
            Err(error) => {
                skill.status = SkillStatus::Error(format!(
                    "Kolejka sprawdzania tagów została przerwana: {}",
                    error
                ));
                skill.update_available = false;
                skill.last_checked = chrono::Utc::now();
                return skill;
            }
        }
    }

    if let Some(remote_url) = skill.remote_url.clone() {
        match GitHubService::check_latest_version(&remote_url).await {
            Ok(Some(release)) => apply_github_release(&mut skill, &release),
            Ok(None) => {
                skill.status = SkillStatus::Error(
                    "Upstream nie udostępnia wersji ani tagu do porównania.".to_string(),
                );
                skill.update_available = false;
            }
            Err(error) => {
                skill.status = SkillStatus::Error(format!(
                    "Nie udało się sprawdzić aktualizacji GitHub: {}",
                    error
                ));
                skill.update_available = false;
            }
        }
    }
    skill.last_checked = chrono::Utc::now();
    skill
}

async fn refresh_branch_upstream(mut skill: SkillMetadata, branch: String) -> SkillMetadata {
    let path = skill.path.clone();
    let branch_for_check = branch.clone();
    match tauri::async_runtime::spawn_blocking(move || {
        let current = GitService::get_head_commit(&path)
            .ok_or_else(|| "Nie można odczytać lokalnego commitu HEAD".to_string())?;
        let remote = GitService::get_remote_branch_commit(&path, &branch_for_check)
            .map_err(|error| error.to_string())?;
        let active_branch = GitService::get_current_branch_name(&path);
        Ok::<_, String>((current, remote, active_branch))
    })
    .await
    {
        Ok(Ok((current, remote, active_branch))) => {
            let branch_mismatch = active_branch.as_deref() != Some(branch.as_str());
            let commit_mismatch = current != remote;
            skill.branch_or_tag = Some(branch.clone());
            skill.latest_version = Some(branch.clone());
            skill.update_available = branch_mismatch || commit_mismatch;
            skill.status = if skill.update_available {
                SkillStatus::UpdateAvailable
            } else {
                SkillStatus::UpToDate
            };
            skill.update_compatibility = Some(if branch_mismatch {
                format!(
                    "Niezgodna gałąź: aktywna {:?}, wymagana {}; aktualizacja wyrówna checkout.",
                    active_branch.unwrap_or_else(|| "detached HEAD".to_string()),
                    branch
                )
            } else if commit_mismatch {
                format!(
                    "Gałąź {} ma nowszy commit upstream; aktualizacja wyrówna lokalny checkout.",
                    branch
                )
            } else {
                format!("Gałąź {} jest zgodna z upstream.", branch)
            });
        }
        Ok(Err(error)) => {
            skill.status = SkillStatus::Error(format!(
                "Nie udało się sprawdzić gałęzi {}: {}",
                branch, error
            ));
            skill.update_available = false;
        }
        Err(error) => {
            skill.status = SkillStatus::Error(format!(
                "Kolejka sprawdzania gałęzi została przerwana: {}",
                error
            ));
            skill.update_available = false;
        }
    }
    skill.last_checked = chrono::Utc::now();
    skill
}

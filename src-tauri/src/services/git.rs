use crate::errors::SkillSyncError;
use git2::{build::CheckoutBuilder, Repository, Status, StatusOptions};
use std::path::Path;

pub struct GitService;

impl GitService {
    pub fn is_git_repository(path: &Path) -> bool {
        path.join(".git").exists() && Repository::open(path).is_ok()
    }

    pub fn get_remote_url(path: &Path) -> Option<String> {
        if !path.join(".git").exists() {
            return None;
        }
        let repo = Repository::open(path).ok()?;
        let remote = repo.find_remote("origin").ok()?;
        remote.url().ok().map(str::to_owned)
    }

    pub fn get_current_ref_name(path: &Path) -> Option<String> {
        if !path.join(".git").exists() {
            return None;
        }
        let repo = Repository::open(path).ok()?;
        let head = repo.head().ok()?;
        if head.is_branch() {
            head.shorthand().ok().map(str::to_owned)
        } else {
            // Detached HEAD or tag
            head.target().map(|oid| oid.to_string()[..7].to_string())
        }
    }

    pub fn get_current_branch_name(path: &Path) -> Option<String> {
        if !path.join(".git").exists() {
            return None;
        }
        let repo = Repository::open(path).ok()?;
        let head = repo.head().ok()?;
        head.is_branch()
            .then(|| head.shorthand().ok().map(str::to_owned))
            .flatten()
    }

    pub fn get_head_commit(path: &Path) -> Option<String> {
        let repo = Repository::open(path).ok()?;
        let commit = repo.head().ok()?.target().map(|oid| oid.to_string());
        commit
    }

    /// Resolves the current SHA of a remote branch without changing the local
    /// checkout. `git ls-remote` honours the user's configured credentials,
    /// which is essential for private agent repositories.
    pub fn get_remote_branch_commit(path: &Path, branch: &str) -> Result<String, SkillSyncError> {
        if branch.trim().is_empty() {
            return Err(SkillSyncError::Git {
                code: -4,
                message: "Nazwa gałęzi nie może być pusta".to_string(),
            });
        }
        let output = std::process::Command::new("git")
            .args([
                "-C",
                path.to_str().unwrap_or("."),
                "ls-remote",
                "--heads",
                "origin",
                branch.trim(),
            ])
            .output()
            .map_err(|error| SkillSyncError::FileSystem(error.to_string()))?;
        if !output.status.success() {
            return Err(SkillSyncError::Git {
                code: output.status.code().unwrap_or(-1),
                message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            });
        }
        let sha = String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_string();
        if sha.is_empty() {
            return Err(SkillSyncError::Git {
                code: -3,
                message: format!("Gałąź '{}' nie istnieje w origin", branch.trim()),
            });
        }
        Ok(sha)
    }

    pub fn fetch_and_checkout_branch(
        path: &Path,
        branch: &str,
        allow_dirty_worktree: bool,
    ) -> Result<(), SkillSyncError> {
        let repo = Repository::open(path)?;
        if !allow_dirty_worktree && !Self::is_worktree_clean(path)? {
            return Err(SkillSyncError::WorktreeDirty);
        }

        let branch = branch.trim();
        let refspec = format!("refs/heads/{branch}:refs/remotes/origin/{branch}");
        let output = std::process::Command::new("git")
            .args([
                "-C",
                path.to_str().unwrap_or("."),
                "fetch",
                "origin",
                &refspec,
            ])
            .output()
            .map_err(|error| SkillSyncError::FileSystem(error.to_string()))?;
        if !output.status.success() {
            return Err(SkillSyncError::Git {
                code: output.status.code().unwrap_or(-1),
                message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            });
        }

        let remote_ref = format!("refs/remotes/origin/{branch}");
        let commit = repo
            .find_reference(&remote_ref)
            .and_then(|reference| reference.peel_to_commit())?;
        let mut checkout = CheckoutBuilder::new();
        if allow_dirty_worktree {
            checkout.force();
        } else {
            checkout.safe();
        }
        repo.checkout_tree(commit.as_object(), Some(&mut checkout))?;

        let local_ref = format!("refs/heads/{branch}");
        if repo.find_reference(&local_ref).is_ok() {
            repo.reference(&local_ref, commit.id(), true, "SkillSync branch update")?;
        } else {
            repo.branch(branch, &commit, false)?;
        }
        repo.set_head(&local_ref)?;
        Ok(())
    }

    pub fn is_worktree_clean(path: &Path) -> Result<bool, SkillSyncError> {
        let repo = Repository::open(path)?;
        let mut opts = StatusOptions::new();
        // Updates change tracked files. Untracked files are commonly created
        // by skill tooling and must not permanently block an update; a real
        // checkout conflict is still rejected by libgit2 below.
        opts.include_untracked(false)
            .include_ignored(false)
            .recurse_untracked_dirs(false);
        let statuses = repo.statuses(Some(&mut opts))?;

        // A deleted tracked file has no local content that a checkout could
        // overwrite. Recreating it from the selected release is safe, and is
        // common when an installed package is left with a removed test file.
        // Any other tracked/index change remains a hard safety stop.
        Ok(statuses
            .iter()
            .all(|entry| entry.status() == Status::WT_DELETED))
    }

    pub fn get_head_tag(path: &Path) -> Option<String> {
        if !path.join(".git").exists() {
            return None;
        }
        let repo = Repository::open(path).ok()?;
        let head = repo.head().ok()?;
        let head_commit = head.peel_to_commit().ok()?;
        let head_id = head_commit.id();

        let tags = repo.tag_names(None).ok()?;
        for tag_name in tags.iter().flatten().flatten() {
            let refname = format!("refs/tags/{}", tag_name);
            if let Ok(reference) = repo.find_reference(&refname) {
                if let Ok(commit) = reference.peel_to_commit() {
                    if commit.id() == head_id {
                        let clean = tag_name.trim_start_matches(['v', 'V']).to_string();
                        return Some(clean);
                    }
                }
            }
        }
        None
    }

    pub fn fetch_and_checkout_tag(
        path: &Path,
        tag_name: &str,
        allow_dirty_worktree: bool,
    ) -> Result<(), SkillSyncError> {
        let repo = Repository::open(path)?;

        // A forced checkout is destructive and is only reachable after an
        // explicit confirmation in the UI. The update transaction still
        // creates its own snapshot before this point.
        if !allow_dirty_worktree && !Self::is_worktree_clean(path)? {
            return Err(SkillSyncError::WorktreeDirty);
        }

        // 1. Fetch tags from remote origin using git CLI (uses system credentials/keys)
        let _ = std::process::Command::new("git")
            .args([
                "-C",
                path.to_str().unwrap_or("."),
                "fetch",
                "--tags",
                "origin",
            ])
            .output();

        // 2. Also attempt fetch via libgit2
        if let Ok(mut remote) = repo.find_remote("origin") {
            let refspec = "+refs/tags/*:refs/tags/*";
            let _ = remote.fetch(&[refspec], None, None);
        }

        // Candidates to resolve (supporting both v1.2.3 and 1.2.3, branch, and remote refs)
        let clean = tag_name.trim_start_matches(['v', 'V']);
        let with_v = format!("v{}", clean);
        let candidates = vec![
            format!("refs/tags/{}", tag_name),
            format!("refs/tags/{}", with_v),
            format!("refs/tags/{}", clean),
            tag_name.to_string(),
            with_v.clone(),
            clean.to_string(),
            format!("origin/{}", tag_name),
            format!("origin/{}", with_v),
            format!("origin/{}", clean),
        ];

        let mut target_commit = None;
        for cand in &candidates {
            if let Ok(reference) = repo.find_reference(cand) {
                if let Ok(commit) = reference.peel_to_commit() {
                    target_commit = Some(commit);
                    break;
                }
            }
            if let Ok(reference) = repo.resolve_reference_from_short_name(cand) {
                if let Ok(commit) = reference.peel_to_commit() {
                    target_commit = Some(commit);
                    break;
                }
            }
            if let Ok(obj) = repo.revparse_single(cand) {
                if let Ok(commit) = obj.peel_to_commit() {
                    target_commit = Some(commit);
                    break;
                }
            }
        }

        let commit = match target_commit {
            Some(c) => c,
            None => {
                // CLI fallback checkout
                for cand in &[&with_v, &clean.to_string(), &tag_name.to_string()] {
                    let mut command = std::process::Command::new("git");
                    command.arg("-C").arg(path).arg("checkout");
                    if allow_dirty_worktree {
                        command.arg("--force");
                    }
                    let res = command.arg(cand).output();
                    if let Ok(out) = res {
                        if out.status.success() {
                            return Ok(());
                        }
                    }
                }
                return Err(SkillSyncError::Git {
                    code: -3,
                    message: format!(
                        "Nie znaleziono tagu ani referencji Git dla '{}' ani '{}'",
                        tag_name, with_v
                    ),
                });
            }
        };

        // Checkout target commit
        let mut checkout = CheckoutBuilder::new();
        if allow_dirty_worktree {
            checkout.force();
        } else {
            checkout.safe();
        }
        repo.checkout_tree(commit.as_object(), Some(&mut checkout))?;
        repo.set_head_detached(commit.id())?;

        Ok(())
    }

    pub fn get_latest_local_tag(path: &Path) -> Option<String> {
        if !path.join(".git").exists() {
            return None;
        }
        let repo = Repository::open(path).ok()?;
        let tags = repo.tag_names(None).ok()?;
        let mut tag_list: Vec<String> =
            tags.iter().flatten().flatten().map(str::to_owned).collect();

        // Sort descending by SemVer
        tag_list.sort_by(|a, b| {
            let clean_a = a.trim_start_matches(['v', 'V']);
            let clean_b = b.trim_start_matches(['v', 'V']);
            match (
                semver::Version::parse(clean_a),
                semver::Version::parse(clean_b),
            ) {
                (Ok(va), Ok(vb)) => vb.cmp(&va),
                _ => b.cmp(a),
            }
        });

        tag_list.into_iter().next()
    }

    pub fn checkout_ref(path: &Path, target_ref: &str) -> Result<(), SkillSyncError> {
        let repo = Repository::open(path)?;

        if !Self::is_worktree_clean(path)? {
            return Err(SkillSyncError::WorktreeDirty);
        }

        // Try fetch first via CLI and git2
        let _ = std::process::Command::new("git")
            .args([
                "-C",
                path.to_str().unwrap_or("."),
                "fetch",
                "--all",
                "--tags",
            ])
            .output();

        if let Ok(mut remote) = repo.find_remote("origin") {
            let _ = remote.fetch(
                &["+refs/heads/*:refs/heads/*", "+refs/tags/*:refs/tags/*"],
                None,
                None,
            );
        }

        let clean = target_ref.trim_start_matches(['v', 'V']);
        let with_v = format!("v{}", clean);
        let candidates = vec![
            target_ref.to_string(),
            format!("refs/tags/{}", target_ref),
            format!("refs/tags/{}", with_v),
            format!("refs/tags/{}", clean),
            format!("origin/{}", target_ref),
            format!("origin/{}", with_v),
            with_v.clone(),
            clean.to_string(),
        ];

        let mut target_commit = None;
        for cand in &candidates {
            if let Ok(reference) = repo.find_reference(cand) {
                if let Ok(commit) = reference.peel_to_commit() {
                    target_commit = Some(commit);
                    break;
                }
            }
            if let Ok(reference) = repo.resolve_reference_from_short_name(cand) {
                if let Ok(commit) = reference.peel_to_commit() {
                    target_commit = Some(commit);
                    break;
                }
            }
            if let Ok(obj) = repo.revparse_single(cand) {
                if let Ok(commit) = obj.peel_to_commit() {
                    target_commit = Some(commit);
                    break;
                }
            }
        }

        let commit = match target_commit {
            Some(c) => c,
            None => {
                // CLI checkout fallback
                for cand in &[target_ref, &with_v, clean] {
                    let res = std::process::Command::new("git")
                        .args(["-C", path.to_str().unwrap_or("."), "checkout", cand])
                        .output();
                    if let Ok(out) = res {
                        if out.status.success() {
                            return Ok(());
                        }
                    }
                }
                return Err(SkillSyncError::Git {
                    code: -3,
                    message: format!("Nie znaleziono referencji Git dla '{}'", target_ref),
                });
            }
        };

        repo.checkout_tree(commit.as_object(), None)?;
        repo.set_head_detached(commit.id())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn committed_repo(name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "skillsync-git-test-{}-{}",
            name,
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        fs::create_dir_all(&path).unwrap();
        let repo = Repository::init(&path).unwrap();
        fs::write(path.join("SKILL.md"), "initial\n").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(Path::new("SKILL.md")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let signature = git2::Signature::now("SkillSync test", "tests@example.invalid").unwrap();
        repo.commit(Some("HEAD"), &signature, &signature, "initial", &tree, &[])
            .unwrap();
        path
    }

    #[test]
    fn untracked_files_do_not_block_an_update() {
        let path = committed_repo("untracked");
        fs::write(path.join("local-notes.txt"), "keep me\n").unwrap();

        assert!(GitService::is_worktree_clean(&path).unwrap());

        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn tracked_file_changes_still_block_an_update() {
        let path = committed_repo("modified");
        fs::write(path.join("SKILL.md"), "locally changed\n").unwrap();

        assert!(!GitService::is_worktree_clean(&path).unwrap());

        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn deleted_tracked_files_do_not_block_an_update() {
        let path = committed_repo("deleted");
        fs::remove_file(path.join("SKILL.md")).unwrap();

        assert!(GitService::is_worktree_clean(&path).unwrap());

        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn detects_the_checked_out_branch_and_head_commit() {
        let path = committed_repo("branch-detection");
        let expected_branch = Repository::open(&path)
            .unwrap()
            .head()
            .unwrap()
            .shorthand()
            .unwrap()
            .to_string();

        assert_eq!(
            GitService::get_current_branch_name(&path),
            Some(expected_branch)
        );
        assert!(GitService::get_head_commit(&path).is_some());

        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn confirmed_force_update_restores_the_selected_tag() {
        let path = committed_repo("force-checkout");
        let repo = Repository::open(&path).unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.tag_lightweight("v1.0.0", head.as_object(), false)
            .unwrap();
        fs::write(path.join("SKILL.md"), "locally changed\n").unwrap();

        assert!(matches!(
            GitService::fetch_and_checkout_tag(&path, "v1.0.0", false),
            Err(SkillSyncError::WorktreeDirty)
        ));

        GitService::fetch_and_checkout_tag(&path, "v1.0.0", true).unwrap();
        let restored = fs::read_to_string(path.join("SKILL.md")).unwrap();
        assert_eq!(restored.replace("\r\n", "\n"), "initial\n");

        let _ = fs::remove_dir_all(path);
    }
}

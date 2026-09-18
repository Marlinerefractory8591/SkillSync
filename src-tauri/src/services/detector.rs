use crate::models::skill::{AgentScope, SkillMetadata, SkillStatus};
use crate::services::git::GitService;
use crate::services::manifest::SkillManifest;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct SkillDetector;

/// Value used only when no local manifest nor Git tag provides a version.
/// It must never be treated as a real release or as evidence that a skill is current.
pub const UNKNOWN_SKILL_VERSION: &str = "unknown";

impl SkillDetector {
    pub fn scan_directories(paths: &[PathBuf]) -> Vec<SkillMetadata> {
        let mut skills_map: std::collections::HashMap<String, SkillMetadata> =
            std::collections::HashMap::new();

        for base_path in paths {
            if !base_path.exists() {
                continue;
            }

            for entry in WalkDir::new(base_path)
                .follow_links(true)
                .max_depth(3)
                .into_iter()
                .filter_entry(|e| !Self::is_ignored_directory(e.file_name()))
                .flatten()
            {
                let p = entry.path();
                if p.is_dir() {
                    if let Some(candidate) = Self::inspect_candidate_directory(p, base_path) {
                        let key = candidate.id.clone();
                        let canonical = fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());

                        if let Some(existing) = skills_map.get_mut(&key) {
                            if !existing.installed_locations.contains(&p.to_path_buf()) {
                                existing.installed_locations.push(p.to_path_buf());
                            }
                            if !existing.installed_locations.contains(&canonical) {
                                existing.installed_locations.push(canonical.clone());
                            }
                            if existing.agent_scope != candidate.agent_scope
                                && existing.agent_scope != AgentScope::Global
                            {
                                existing.agent_scope = AgentScope::Global;
                            }
                            if existing.remote_url.is_none() && candidate.remote_url.is_some() {
                                existing.remote_url = candidate.remote_url;
                            }
                            if existing.compatibility.is_none() && candidate.compatibility.is_some()
                            {
                                existing.compatibility = candidate.compatibility;
                            }
                        } else {
                            let mut new_skill = candidate;
                            if !new_skill.installed_locations.contains(&canonical) {
                                new_skill.installed_locations.push(canonical);
                            }
                            skills_map.insert(key, new_skill);
                        }
                    }
                }
            }
        }

        let mut skills: Vec<SkillMetadata> = skills_map.into_values().collect();
        skills.sort_by_key(|skill| skill.name.to_lowercase());
        skills
    }

    fn is_ignored_directory(name: &std::ffi::OsStr) -> bool {
        let s = name.to_string_lossy();
        s == ".git" || s == "node_modules" || s == "target" || s == "dist"
    }

    fn inspect_candidate_directory(dir: &Path, base_monitored: &Path) -> Option<SkillMetadata> {
        let skill_json = dir.join("skill.json");
        let skill_md = dir.join("SKILL.md");
        let package_json = dir.join("package.json");

        let folder_name = dir.file_name()?.to_string_lossy().to_string();

        // 1. Check skill.json
        if skill_json.exists() {
            if let Ok(content) = fs::read_to_string(&skill_json) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                    let name = v
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or(&folder_name)
                        .to_string();
                    let version = v
                        .get("version")
                        .and_then(|ver| ver.as_str())
                        .unwrap_or(UNKNOWN_SKILL_VERSION)
                        .to_string();
                    let desc = v
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string();
                    let author = v
                        .get("author")
                        .and_then(|a| a.as_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    let scope =
                        Self::infer_scope(v.get("scope").and_then(|s| s.as_str()), base_monitored);

                    let is_git = GitService::is_git_repository(dir);
                    let fm_dummy = FrontmatterMeta::default();
                    let remote_url = Self::resolve_remote_url(dir, &fm_dummy);
                    let branch_or_tag = GitService::get_current_ref_name(dir);
                    let compatibility = Self::infer_compatibility(dir, &scope, &fm_dummy);

                    return Some(SkillMetadata {
                        item_type: crate::models::skill::ManagedItemType::Skill,
                        id: format!("skill-{}", name.replace(' ', "-").to_lowercase()),
                        name,
                        description: desc,
                        current_version: version,
                        latest_version: None,
                        author,
                        path: dir.to_path_buf(),
                        is_git_repo: is_git,
                        remote_url,
                        branch_or_tag,
                        detected_branch: GitService::get_current_branch_name(dir),
                        branch_override: None,
                        agent_scope: scope,
                        status: SkillStatus::UpToDate,
                        update_available: false,
                        changelog: None,
                        dependencies: vec![],
                        permissions: vec![],
                        last_checked: chrono::Utc::now(),
                        compatibility: Some(compatibility),
                        update_compatibility: None,
                        installed_locations: vec![dir.to_path_buf()],
                    });
                }
            }
        }

        // 2. Check SKILL.md with YAML frontmatter
        if skill_md.exists() {
            let fm = if let Ok(content) = fs::read_to_string(&skill_md) {
                Self::parse_yaml_frontmatter(&content)
            } else {
                FrontmatterMeta::default()
            };

            let name = fm.name.clone().unwrap_or_else(|| folder_name.clone());
            let desc = fm
                .description
                .clone()
                .unwrap_or_else(|| "Skill with Markdown documentation".to_string());
            let author = fm.author.clone().unwrap_or_else(|| "Community".to_string());
            let scope = Self::infer_scope(fm.scope.as_deref(), base_monitored);

            let is_git = GitService::is_git_repository(dir);
            let remote_url = Self::resolve_remote_url(dir, &fm);
            let branch_or_tag = GitService::get_current_ref_name(dir);
            let compatibility = Self::infer_compatibility(dir, &scope, &fm);

            // If git has tags, prefer git tag if no explicit version in SKILL.md
            let version = fm
                .version
                .clone()
                .or_else(|| GitService::get_head_tag(dir))
                .unwrap_or_else(|| UNKNOWN_SKILL_VERSION.to_string());

            return Some(SkillMetadata {
                item_type: crate::models::skill::ManagedItemType::Skill,
                id: format!("skill-{}", name.replace(' ', "-").to_lowercase()),
                name,
                description: desc,
                current_version: version,
                latest_version: None,
                author,
                path: dir.to_path_buf(),
                is_git_repo: is_git,
                remote_url,
                branch_or_tag,
                detected_branch: GitService::get_current_branch_name(dir),
                branch_override: None,
                agent_scope: scope,
                status: SkillStatus::UpToDate,
                update_available: false,
                changelog: None,
                dependencies: vec![],
                permissions: vec![],
                last_checked: chrono::Utc::now(),
                compatibility: Some(compatibility),
                update_compatibility: None,
                installed_locations: vec![dir.to_path_buf()],
            });
        }

        // 3. Check package.json
        if package_json.exists() {
            if let Ok(content) = fs::read_to_string(&package_json) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                    if SkillManifest::is_explicit_package_skill(dir) {
                        let name = v
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or(&folder_name)
                            .to_string();
                        let raw_version = v
                            .get("version")
                            .and_then(|ver| ver.as_str())
                            .unwrap_or(UNKNOWN_SKILL_VERSION)
                            .to_string();
                        let version = raw_version.trim_start_matches(['v', 'V']).to_string();
                        let desc = v
                            .get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("Node-based AI skill")
                            .to_string();
                        let author = v
                            .get("author")
                            .and_then(|a| a.as_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        let scope = Self::infer_scope(None, base_monitored);

                        let is_git = GitService::is_git_repository(dir);
                        let fm_dummy = FrontmatterMeta::default();
                        let remote_url = Self::resolve_remote_url(dir, &fm_dummy);
                        let branch_or_tag = GitService::get_current_ref_name(dir);
                        let compatibility = Self::infer_compatibility(dir, &scope, &fm_dummy);

                        return Some(SkillMetadata {
                            item_type: crate::models::skill::ManagedItemType::Skill,
                            id: format!("skill-{}", name.replace(' ', "-").to_lowercase()),
                            name,
                            description: desc,
                            current_version: version,
                            latest_version: None,
                            author,
                            path: dir.to_path_buf(),
                            is_git_repo: is_git,
                            remote_url,
                            branch_or_tag,
                            detected_branch: GitService::get_current_branch_name(dir),
                            branch_override: None,
                            agent_scope: scope,
                            status: SkillStatus::UpToDate,
                            update_available: false,
                            changelog: None,
                            dependencies: vec![],
                            permissions: vec![],
                            last_checked: chrono::Utc::now(),
                            compatibility: Some(compatibility),
                            update_compatibility: None,
                            installed_locations: vec![dir.to_path_buf()],
                        });
                    }
                }
            }
        }

        None
    }

    fn infer_scope(manifest_scope: Option<&str>, base_path: &Path) -> AgentScope {
        if let Some(s) = manifest_scope {
            match s.to_lowercase().as_str() {
                "codex" => return AgentScope::Codex,
                "claude" => return AgentScope::Claude,
                "cursor" => return AgentScope::Cursor,
                "antigravity" => return AgentScope::Antigravity,
                "global" => return AgentScope::Global,
                other => return AgentScope::Custom(other.to_string()),
            }
        }

        let p_str = base_path.to_string_lossy().to_lowercase();
        if p_str.contains(".codex") {
            AgentScope::Codex
        } else if p_str.contains(".claude") {
            AgentScope::Claude
        } else if p_str.contains(".cursor") {
            AgentScope::Cursor
        } else if p_str.contains("antigravity") {
            AgentScope::Antigravity
        } else {
            AgentScope::Global
        }
    }

    fn infer_compatibility(dir: &Path, scope: &AgentScope, fm: &FrontmatterMeta) -> String {
        if let Some(ref comp) = fm.compatibility {
            return comp.clone();
        }

        let mut parts = Vec::new();
        match scope {
            AgentScope::Codex => parts.push("OpenAI Codex"),
            AgentScope::Claude => parts.push("Claude Code (v1.0+)"),
            AgentScope::Cursor => parts.push("Cursor IDE"),
            AgentScope::Antigravity => parts.push("Google Antigravity"),
            AgentScope::Global => parts.push("Wszystkie agenty (Uniwersalny)"),
            AgentScope::Custom(s) => parts.push(s.as_str()),
        }

        if dir.join("pyproject.toml").exists()
            || dir.join("requirements.txt").exists()
            || dir.join("scripts").exists()
        {
            parts.push("Python >= 3.10");
        } else if dir.join("package.json").exists() {
            parts.push("Node.js >= 18");
        }

        parts.join(" • ")
    }

    fn resolve_remote_url(dir: &Path, fm: &FrontmatterMeta) -> Option<String> {
        // 1. Direct git repository
        if let Some(url) = GitService::get_remote_url(dir) {
            return Some(url);
        }

        // 2. Parent git repository
        if let Some(parent) = dir.parent() {
            if let Some(url) = GitService::get_remote_url(parent) {
                return Some(url);
            }
        }

        // 3. Frontmatter explicit repository
        if let Some(ref repo) = fm.repository {
            if repo.starts_with("http") || repo.starts_with("git@") {
                return Some(repo.clone());
            }
            if repo.contains('/') && !repo.contains(' ') {
                return Some(format!(
                    "https://github.com/{}",
                    repo.trim_start_matches('/')
                ));
            }
        }

        // 4. Check LICENSE.txt, LICENSE, LICENSE.md
        for lic_name in &["LICENSE.txt", "LICENSE", "LICENSE.md"] {
            let lic_path = dir.join(lic_name);
            if lic_path.exists() {
                if let Ok(content) = fs::read_to_string(&lic_path) {
                    if let Some(url) = Self::extract_github_url(&content) {
                        return Some(url);
                    }
                }
            }
        }

        // 5. Check pyproject.toml in dir or dir.parent()
        for p in &[
            dir.join("pyproject.toml"),
            dir.parent()
                .map(|p| p.join("pyproject.toml"))
                .unwrap_or_default(),
        ] {
            if p.exists() {
                if let Ok(content) = fs::read_to_string(p) {
                    if let Some(url) = Self::extract_github_url(&content) {
                        return Some(url);
                    }
                }
            }
        }

        // 6. Check package.json in dir or dir.parent()
        for p in &[
            dir.join("package.json"),
            dir.parent()
                .map(|p| p.join("package.json"))
                .unwrap_or_default(),
        ] {
            if p.exists() {
                if let Ok(content) = fs::read_to_string(p) {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(repo) = v.get("repository") {
                            if let Some(s) = repo.as_str() {
                                if s.starts_with("http") {
                                    return Some(s.to_string());
                                } else if s.contains('/') {
                                    return Some(format!("https://github.com/{}", s));
                                }
                            } else if let Some(url) = repo.get("url").and_then(|u| u.as_str()) {
                                return Some(url.to_string());
                            }
                        }
                    }
                    if let Some(url) = Self::extract_github_url(&content) {
                        return Some(url);
                    }
                }
            }
        }

        // 7. Check SKILL.md or README.md
        for doc_name in &["SKILL.md", "README.md"] {
            let doc_path = dir.join(doc_name);
            if doc_path.exists() {
                if let Ok(content) = fs::read_to_string(&doc_path) {
                    if let Some(url) = Self::extract_github_url(&content) {
                        return Some(url);
                    }
                }
            }
        }

        None
    }

    fn extract_github_url(content: &str) -> Option<String> {
        let marker = "https://github.com/";
        let mut start_idx = 0;
        while let Some(pos) = content[start_idx..].find(marker) {
            let full_pos = start_idx + pos;
            let after = &content[full_pos..];
            let url_candidate: String = after
                .chars()
                .take_while(|c| {
                    c.is_alphanumeric()
                        || *c == ':'
                        || *c == '/'
                        || *c == '-'
                        || *c == '_'
                        || *c == '.'
                })
                .collect();

            let trimmed = url_candidate.trim_end_matches('.').trim_end_matches('/');
            let parts: Vec<&str> = trimmed.split('/').collect();
            // Expected: ["https:", "", "github.com", "owner", "repo"]
            if parts.len() >= 5 {
                let owner = parts[3];
                let repo = parts[4].trim_end_matches(".git");
                if !owner.is_empty()
                    && !repo.is_empty()
                    && owner != "user-attachments"
                    && owner != "OWNER"
                    && !repo.contains('#')
                    && !repo.contains('?')
                {
                    return Some(format!("https://github.com/{}/{}", owner, repo));
                }
            }
            start_idx = full_pos + marker.len();
        }
        None
    }

    fn parse_yaml_frontmatter(content: &str) -> FrontmatterMeta {
        let mut meta = FrontmatterMeta::default();
        let trimmed = content.trim();
        if !trimmed.starts_with("---") {
            return meta;
        }

        let rest = &trimmed[3..];
        if let Some(end_pos) = rest.find("---") {
            let yaml_block = &rest[..end_pos];
            for line in yaml_block.lines() {
                let line = line.trim();
                if let Some((key, val)) = line.split_once(':') {
                    let key = key.trim().to_lowercase();
                    let val = val.trim().trim_matches('"').trim_matches('\'').to_string();
                    if !val.is_empty() {
                        match key.as_str() {
                            "name" => meta.name = Some(val),
                            "version" => meta.version = Some(val),
                            "description" => meta.description = Some(val),
                            "author" => meta.author = Some(val),
                            "scope" => meta.scope = Some(val),
                            "repository" | "repo" | "url" | "homepage" | "github" | "source" => {
                                meta.repository = Some(val)
                            }
                            "compatibility" | "targets" | "target" | "requires" => {
                                meta.compatibility = Some(val)
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        meta
    }
}

#[derive(Default)]
struct FrontmatterMeta {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    author: Option<String>,
    scope: Option<String>,
    repository: Option<String>,
    compatibility: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "skillsync-detector-{name}-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ))
    }

    fn write_file(path: &Path, content: &str) {
        fs::create_dir_all(path.parent().expect("fixture file has a parent")).unwrap();
        fs::write(path, content).unwrap();
    }

    #[test]
    fn detects_claude_skill_scope_from_a_fixture() {
        let root = fixture_root("claude-scope");
        let skills_dir = root.join(".claude/skills");
        let skill_dir = skills_dir.join("example-skill");
        write_file(
            &skill_dir.join("SKILL.md"),
            "---\nname: example-skill\nversion: 1.0.0\n---\n",
        );

        let skills = SkillDetector::scan_directories(&[skills_dir]);

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].agent_scope, AgentScope::Claude);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn scan_is_safe_for_empty_and_missing_monitored_paths() {
        let missing = fixture_root("missing").join("does-not-exist");

        assert!(SkillDetector::scan_directories(&[]).is_empty());
        assert!(SkillDetector::scan_directories(&[missing]).is_empty());
    }

    #[test]
    fn test_detects_codex_skill_scope() {
        let root = std::env::temp_dir().join(format!(
            "skillsync-codex-detection-test-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let skills_dir = root.join(".codex/skills");
        let skill_dir = skills_dir.join("example-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: example-skill\nversion: 1.0.0\n---\n",
        )
        .unwrap();

        let skills = SkillDetector::scan_directories(&[skills_dir]);

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].agent_scope, AgentScope::Codex);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_update_skill_md_version() {
        let temp_dir = std::env::temp_dir().join(format!(
            "skillsync-test-skill-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let _ = fs::create_dir_all(&temp_dir);
        let skill_md = temp_dir.join("SKILL.md");

        let sample_content = r#"---
name: test-skill
description: "A test skill"
metadata:
  author: TestAuthor
  version: "1.0.0"
---

# Test Content
"#;
        fs::write(&skill_md, sample_content).unwrap();

        // Update version to 2.3.1
        crate::services::orchestrator::UpdateOrchestrator::update_skill_md_version(
            &temp_dir, "2.3.1",
        )
        .unwrap();

        let updated_content = fs::read_to_string(&skill_md).unwrap();
        println!("Updated content:\n{}", updated_content);
        assert!(updated_content.contains("version: \"2.3.1\""));

        // Verify parsing back with SkillDetector
        let meta = SkillDetector::parse_yaml_frontmatter(&updated_content);
        assert_eq!(meta.version.as_deref(), Some("2.3.1"));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn detects_skill_markdown_manifest() {
        let root = fixture_root("markdown");
        let skill_dir = root.join("skills/markdown-skill");
        write_file(
            &skill_dir.join("SKILL.md"),
            "---\nname: markdown-skill\nversion: 1.2.3\n---\n# Skill\n",
        );

        let skills = SkillDetector::scan_directories(&[root.join("skills")]);
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "markdown-skill");
        assert_eq!(skills[0].current_version, "1.2.3");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn detects_skill_json_manifest() {
        let root = fixture_root("json");
        let skill_dir = root.join("skills/json-skill");
        write_file(
            &skill_dir.join("skill.json"),
            r#"{"name":"json-skill","version":"2.0.0","description":"Fixture"}"#,
        );

        let skills = SkillDetector::scan_directories(&[root.join("skills")]);
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "json-skill");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn marks_a_manifest_without_version_as_unknown() {
        let root = fixture_root("unknown-version");
        let skill_dir = root.join("skills/unknown-skill");
        write_file(
            &skill_dir.join("skill.json"),
            r#"{"name":"unknown-skill","description":"Fixture"}"#,
        );

        let skills = SkillDetector::scan_directories(&[root.join("skills")]);
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].current_version, UNKNOWN_SKILL_VERSION);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn detects_explicit_skill_package_manifest() {
        let root = fixture_root("package-skill");
        let skill_dir = root.join("skills/package-skill");
        write_file(
            &skill_dir.join("package.json"),
            r#"{"name":"package-skill","version":"v3.1.4","skill":{"entry":"index.js"}}"#,
        );

        let skills = SkillDetector::scan_directories(&[root.join("skills")]);
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "package-skill");
        assert_eq!(skills[0].current_version, "3.1.4");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn detects_explicit_ai_skill_package_manifest() {
        let root = fixture_root("ai-package-skill");
        let skill_dir = root.join("skills/ai-package-skill");
        write_file(
            &skill_dir.join("package.json"),
            r#"{"name":"ai-package-skill","version":"1.0.0","ai-skill":true}"#,
        );

        let skills = SkillDetector::scan_directories(&[root.join("skills")]);
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "ai-package-skill");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn ignores_regular_packages_below_a_monitored_skills_directory() {
        let root = fixture_root("false-positives");
        let skills_root = root.join(".agents/skills");
        for relative in [
            "hyperframes/packages/aws-lambda",
            "agent-browser/docs",
            "ui-ux-pro-max-skill/gallery",
        ] {
            write_file(
                &skills_root.join(relative).join("package.json"),
                r#"{"name":"ordinary-project-file","version":"1.0.0"}"#,
            );
        }

        let skills = SkillDetector::scan_directories(&[skills_root]);
        assert!(
            skills.is_empty(),
            "ordinary nested package.json files are not skills"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn ignores_malformed_or_disabled_package_manifests() {
        let root = fixture_root("invalid-package");
        let skills_root = root.join("skills");
        write_file(&skills_root.join("malformed/package.json"), "{not-json");
        write_file(
            &skills_root.join("disabled/package.json"),
            r#"{"name":"disabled","skill":false}"#,
        );

        let skills = SkillDetector::scan_directories(&[skills_root]);
        assert!(skills.is_empty());

        let _ = fs::remove_dir_all(root);
    }
}

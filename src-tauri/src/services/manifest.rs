use std::fs;
use std::path::Path;

/// A supported, updateable skill manifest. Ordinary project files are never
/// considered manifests: a monitored directory can contain many repositories
/// and package.json files that are unrelated to agent skills.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillManifestKind {
    Markdown,
    Json,
    ExplicitPackage,
}

pub struct SkillManifest;

impl SkillManifest {
    /// Returns the manifest type or a user-facing reason why the directory is
    /// not safe to update. A malformed skill.json is rejected even when a
    /// sibling SKILL.md exists: silently accepting it would leave an invalid
    /// installed skill after the transaction.
    pub fn validate(dir: &Path) -> Result<SkillManifestKind, String> {
        let skill_json = dir.join("skill.json");
        if skill_json.exists() {
            let content = fs::read_to_string(&skill_json)
                .map_err(|error| format!("Nie można odczytać skill.json: {error}"))?;
            serde_json::from_str::<serde_json::Value>(&content)
                .map_err(|_| "skill.json nie zawiera poprawnego JSON".to_string())?;
            return Ok(SkillManifestKind::Json);
        }

        if dir.join("SKILL.md").is_file() {
            return Ok(SkillManifestKind::Markdown);
        }

        if Self::is_explicit_package_skill(dir) {
            return Ok(SkillManifestKind::ExplicitPackage);
        }

        Err(
            "Brak obsługiwanego manifestu SKILL.md, skill.json lub jawnego metadanych package.json"
                .to_string(),
        )
    }

    /// package.json is a skill manifest only when its author explicitly marks
    /// it with `skill` or `ai-skill`. Being below a directory named `skills`
    /// is deliberately not enough: package workspaces, docs and galleries are
    /// common false positives.
    pub fn is_explicit_package_skill(dir: &Path) -> bool {
        let package_json = dir.join("package.json");
        let Ok(content) = fs::read_to_string(package_json) else {
            return false;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
            return false;
        };

        ["skill", "ai-skill"].iter().any(|field| {
            value
                .get(*field)
                .is_some_and(Self::is_enabled_skill_metadata)
        })
    }

    fn is_enabled_skill_metadata(value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::Null | serde_json::Value::Bool(false) => false,
            serde_json::Value::String(value) => !value.trim().is_empty(),
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_dir(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "skillsync-manifest-{name}-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ))
    }

    #[test]
    fn validates_markdown_manifest() {
        let dir = fixture_dir("markdown");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("SKILL.md"), "# Valid skill\n").unwrap();

        assert_eq!(
            SkillManifest::validate(&dir),
            Ok(SkillManifestKind::Markdown)
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn validates_only_explicit_package_metadata() {
        let dir = fixture_dir("package");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("package.json"),
            r#"{"name":"fixture","ai-skill":{"format":"agent-skills"}}"#,
        )
        .unwrap();

        assert_eq!(
            SkillManifest::validate(&dir),
            Ok(SkillManifestKind::ExplicitPackage)
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_ordinary_package_json() {
        let dir = fixture_dir("ordinary-package");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("package.json"), r#"{"name":"docs"}"#).unwrap();

        assert!(SkillManifest::validate(&dir).is_err());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_malformed_skill_json() {
        let dir = fixture_dir("malformed-json");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("skill.json"), "not-json").unwrap();
        fs::write(dir.join("SKILL.md"), "# This cannot mask malformed JSON\n").unwrap();

        assert!(SkillManifest::validate(&dir).is_err());
        let _ = fs::remove_dir_all(dir);
    }
}

use crate::errors::SkillSyncError;
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct McpService;

impl McpService {
    pub fn is_laravel_boost_project(root: &Path) -> bool {
        root.join("composer.lock").is_file()
            && Self::composer_requires(root, "laravel/boost")
            && Self::laravel_boost_version(root).is_some()
    }

    pub fn laravel_boost_version(root: &Path) -> Option<String> {
        let lock = Self::read_json(root.join("composer.lock"))?;
        for section in ["packages", "packages-dev"] {
            let Some(packages) = lock.get(section).and_then(serde_json::Value::as_array) else {
                continue;
            };
            if let Some(version) = packages.iter().find_map(|package| {
                (package.get("name").and_then(serde_json::Value::as_str) == Some("laravel/boost"))
                    .then(|| package.get("version").and_then(serde_json::Value::as_str))
                    .flatten()
            }) {
                return Some(version.trim_start_matches(['v', 'V']).to_string());
            }
        }
        None
    }

    /// Update a Composer-managed Laravel Boost installation only after the
    /// caller made a full project snapshot. Composer owns the lockfile and
    /// vendor tree, so this operation is never inferred for generic MCP files.
    pub fn update_laravel_boost(root: &Path) -> Result<(), SkillSyncError> {
        if !Self::is_laravel_boost_project(root) {
            return Err(SkillSyncError::UnsupportedUpdateMethod(format!(
                "{} nie jest projektem Composer z composer.lock i wymaganiem laravel/boost",
                root.display()
            )));
        }

        Self::run_composer(root, ["validate", "--no-check-publish"])?;
        Self::run_composer(
            root,
            [
                "update",
                "laravel/boost",
                "--with-all-dependencies",
                "--no-interaction",
                "--no-progress",
            ],
        )?;
        Self::run_composer(root, ["validate", "--no-check-publish"])?;

        if Self::has_test_script(root) {
            Self::run_composer(root, ["run", "test", "--no-interaction"])?;
        }

        if Self::laravel_boost_version(root).is_none() {
            return Err(SkillSyncError::IntegrityCheckFailed(
                "composer.lock nie zawiera laravel/boost po aktualizacji".to_string(),
            ));
        }
        Ok(())
    }

    fn composer_requires(root: &Path, package: &str) -> bool {
        let Some(composer) = Self::read_json(root.join("composer.json")) else {
            return false;
        };
        ["require", "require-dev"].iter().any(|section| {
            composer
                .get(*section)
                .and_then(serde_json::Value::as_object)
                .is_some_and(|requirements| requirements.contains_key(package))
        })
    }

    fn has_test_script(root: &Path) -> bool {
        Self::read_json(root.join("composer.json"))
            .and_then(|composer| composer.get("scripts").cloned())
            .and_then(|scripts| scripts.get("test").cloned())
            .is_some()
    }

    fn run_composer<const N: usize>(
        root: &Path,
        arguments: [&str; N],
    ) -> Result<(), SkillSyncError> {
        let output = Command::new("composer")
            .args(arguments)
            .current_dir(root)
            .output()
            .map_err(|error| {
                SkillSyncError::UnsupportedUpdateMethod(format!(
                    "Nie można uruchomić Composer: {error}. Zainstaluj Composer i spróbuj ponownie."
                ))
            })?;
        if output.status.success() {
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let detail = if stderr.is_empty() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            stderr
        };
        Err(SkillSyncError::FileSystem(format!(
            "Composer zakończył aktualizację Laravel Boost błędem: {detail}"
        )))
    }

    fn read_json(path: impl AsRef<Path>) -> Option<serde_json::Value> {
        fs::read_to_string(path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_dir(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "skillsync-mcp-{name}-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ))
    }

    #[test]
    fn detects_a_composer_project_with_laravel_boost_and_reads_its_locked_version() {
        let dir = fixture_dir("boost-project");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("composer.json"),
            r#"{"require":{"laravel/boost":"^1.0"},"scripts":{"test":"pest"}}"#,
        )
        .unwrap();
        fs::write(
            dir.join("composer.lock"),
            r#"{"packages":[{"name":"laravel/boost","version":"v1.4.2"}]}"#,
        )
        .unwrap();

        assert!(McpService::is_laravel_boost_project(&dir));
        assert_eq!(
            McpService::laravel_boost_version(&dir).as_deref(),
            Some("1.4.2")
        );
        assert!(McpService::has_test_script(&dir));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_a_composer_project_without_a_locked_laravel_boost_dependency() {
        let dir = fixture_dir("not-boost");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("composer.json"),
            r#"{"require":{"laravel/framework":"^12"}}"#,
        )
        .unwrap();
        fs::write(dir.join("composer.lock"), r#"{"packages":[]}"#).unwrap();

        assert!(!McpService::is_laravel_boost_project(&dir));
        let _ = fs::remove_dir_all(dir);
    }
}

use crate::errors::SkillSyncError;
use crate::services::managed_manifest::ManagedManifest;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The official Claude Code CLI owns its cache and registry. Updating its
/// cache directly would leave `installed_plugins.json` inconsistent, so this
/// adapter delegates only to `claude plugin update` after proving that the
/// selected cache directory is an active registry entry.
#[derive(Debug, Clone)]
pub struct ClaudePluginInstallation {
    pub identifier: String,
    pub scope: String,
    pub install_path: PathBuf,
    pub plugins_root: PathBuf,
}

pub struct ClaudePluginService;

impl ClaudePluginService {
    pub fn plugin_version(path: &Path) -> Option<String> {
        fs::read_to_string(path.join(".claude-plugin/plugin.json"))
            .ok()
            .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
            .and_then(|manifest| {
                manifest
                    .get("version")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string)
            })
    }

    pub fn installation_for_path(path: &Path) -> Option<ClaudePluginInstallation> {
        let plugins_root = Self::plugins_root_for(path)?;
        let registry = Self::read_registry(&plugins_root)?;
        let plugins = registry.get("plugins")?.as_object()?;
        let canonical_path = fs::canonicalize(path).ok()?;

        for (identifier, entries) in plugins {
            for entry in entries.as_array()? {
                let install_path = entry.get("installPath")?.as_str()?;
                let install_path = PathBuf::from(install_path);
                if fs::canonicalize(&install_path).ok().as_ref() == Some(&canonical_path) {
                    return Some(ClaudePluginInstallation {
                        identifier: identifier.to_string(),
                        scope: entry
                            .get("scope")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("user")
                            .to_string(),
                        install_path,
                        plugins_root,
                    });
                }
            }
        }
        None
    }

    pub fn is_claude_cache_path(path: &Path) -> bool {
        Self::plugins_root_for(path).is_some_and(|root| path.starts_with(root.join("cache")))
    }

    pub fn update(path: &Path) -> Result<Vec<PathBuf>, SkillSyncError> {
        let installation = Self::installation_for_path(path).ok_or_else(|| {
            SkillSyncError::UnsupportedUpdateMethod(format!(
                "{} nie jest aktywną instalacją z rejestru Claude Code; odśwież skanowanie, aby pominąć nieużywany cache",
                path.display()
            ))
        })?;

        let cli = Self::resolve_cli().ok_or_else(|| {
            SkillSyncError::UnsupportedUpdateMethod(
                "Nie znaleziono Claude Code CLI. Ustaw SKILLSYNC_CLAUDE_CLI albo zainstaluj Claude Code (np. `~/.local/bin/claude`) i uruchom skanowanie ponownie."
                    .to_string(),
            )
        })?;
        let mut command = Command::new(&cli);
        if let Some(path) = Self::runtime_path(&cli) {
            command.env("PATH", path);
        }
        let output = command
            .args([
                "plugin",
                "update",
                &installation.identifier,
                "--scope",
                &installation.scope,
                "--yes",
                "--json",
            ])
            .output()
            .map_err(|error| {
                SkillSyncError::UnsupportedUpdateMethod(format!(
                    "Nie można uruchomić Claude Code CLI ({}) : {error}. Ustaw SKILLSYNC_CLAUDE_CLI albo zainstaluj Claude Code i spróbuj ponownie.",
                    cli.display()
                ))
            })?;
        if !output.status.success() {
            let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let detail = if detail.is_empty() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                detail
            };
            return Err(SkillSyncError::FileSystem(format!(
                "Claude Code nie zaktualizował pluginu {}: {detail}",
                installation.identifier
            )));
        }

        let locations = Self::active_locations(
            &installation.plugins_root,
            &installation.identifier,
            &installation.scope,
        );
        if locations.is_empty()
            || locations
                .iter()
                .any(|location| ManagedManifest::validate_plugin(location).is_err())
        {
            return Err(SkillSyncError::IntegrityCheckFailed(format!(
                "Claude Code nie pozostawił poprawnego manifestu pluginu {} po aktualizacji",
                installation.identifier
            )));
        }
        Ok(locations)
    }

    fn resolve_cli() -> Option<PathBuf> {
        if let Some(configured) = std::env::var_os("SKILLSYNC_CLAUDE_CLI") {
            let configured = PathBuf::from(configured);
            if configured.is_file() {
                return Some(configured);
            }
        }

        let executable_names: &[&OsStr] = if cfg!(windows) {
            &[
                OsStr::new("claude.exe"),
                OsStr::new("claude.cmd"),
                OsStr::new("claude"),
            ]
        } else {
            &[OsStr::new("claude")]
        };
        let path = std::env::var_os("PATH").unwrap_or_default();
        for directory in std::env::split_paths(&path) {
            for name in executable_names {
                let candidate = directory.join(name);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }

        let home = dirs::home_dir()?;
        let mut candidates = vec![
            home.join(".local/bin/claude"),
            home.join(".claude/local/claude"),
            home.join(".npm-global/bin/claude"),
            PathBuf::from("/opt/homebrew/bin/claude"),
            PathBuf::from("/usr/local/bin/claude"),
        ];
        if let Ok(entries) = fs::read_dir(home.join(".nvm/versions/node")) {
            let mut node_bins: Vec<PathBuf> = entries
                .flatten()
                .map(|entry| entry.path().join("bin"))
                .filter(|path| path.is_dir())
                .collect();
            node_bins.sort_by(|left, right| right.cmp(left));
            candidates.extend(node_bins.into_iter().map(|bin| bin.join("claude")));
        }
        candidates.into_iter().find(|candidate| candidate.is_file())
    }

    fn runtime_path(cli: &Path) -> Option<OsString> {
        let mut directories: Vec<PathBuf> =
            std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).collect();
        if let Some(parent) = cli.parent() {
            if !directories.iter().any(|directory| directory == parent) {
                directories.insert(0, parent.to_path_buf());
            }
        }
        let home = dirs::home_dir()?;
        if let Ok(entries) = fs::read_dir(home.join(".nvm/versions/node")) {
            for bin in entries.flatten().map(|entry| entry.path().join("bin")) {
                if bin.is_dir() && !directories.iter().any(|directory| directory == &bin) {
                    directories.push(bin);
                }
            }
        }
        std::env::join_paths(directories).ok()
    }

    fn active_locations(plugins_root: &Path, identifier: &str, scope: &str) -> Vec<PathBuf> {
        let Some(registry) = Self::read_registry(plugins_root) else {
            return Vec::new();
        };
        registry
            .get("plugins")
            .and_then(serde_json::Value::as_object)
            .and_then(|plugins| plugins.get(identifier))
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter(|entry| entry.get("scope").and_then(serde_json::Value::as_str) == Some(scope))
            .filter_map(|entry| entry.get("installPath").and_then(serde_json::Value::as_str))
            .map(PathBuf::from)
            .collect()
    }

    fn plugins_root_for(path: &Path) -> Option<PathBuf> {
        path.ancestors().find_map(|ancestor| {
            (ancestor.file_name()?.to_string_lossy() == "plugins"
                && ancestor.parent()?.file_name()?.to_string_lossy() == ".claude")
                .then(|| ancestor.to_path_buf())
        })
    }

    fn read_registry(plugins_root: &Path) -> Option<serde_json::Value> {
        fs::read_to_string(plugins_root.join("installed_plugins.json"))
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_only_the_active_claude_cache_installation() {
        let home = std::env::temp_dir().join(format!(
            "skillsync-claude-plugin-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let plugins = home.join(".claude/plugins");
        let active = plugins.join("cache/marketplace/demo/1.0.0");
        let stale = plugins.join("cache/marketplace/demo/2.0.0");
        fs::create_dir_all(&active).unwrap();
        fs::create_dir_all(&stale).unwrap();
        fs::write(
            plugins.join("installed_plugins.json"),
            serde_json::json!({
                "plugins": {
                    "demo@marketplace": [{
                        "scope": "user",
                        "installPath": active,
                    }]
                }
            })
            .to_string(),
        )
        .unwrap();

        assert!(ClaudePluginService::installation_for_path(&active).is_some());
        assert!(ClaudePluginService::installation_for_path(&stale).is_none());
        assert!(ClaudePluginService::is_claude_cache_path(&stale));
        let _ = fs::remove_dir_all(home);
    }
}

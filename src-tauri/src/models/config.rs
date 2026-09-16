use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub paths: PathsConfig,
    pub updates: UpdatesConfig,
    pub notifications: NotificationsConfig,
    pub appearance: AppearanceConfig,
    pub advanced: AdvancedConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        Self {
            general: GeneralConfig {
                language: "pl".into(),
                launch_at_login: false,
                minimize_to_tray: true,
                check_app_updates: true,
            },
            paths: PathsConfig {
                monitored: PathsConfig::discover_default_paths(),
                default_install_directory: home.join(".agents/skills"),
            },
            updates: UpdatesConfig {
                auto_check_frequency: "every_6_hours".into(),
                auto_install: "ask".into(),
                concurrency_limit: 4,
                backup_retention_days: 14,
                allow_prerelease: false,
            },
            notifications: NotificationsConfig {
                enabled: true,
                on_update_found: true,
                on_update_success: true,
                on_update_failure: true,
                sound: true,
            },
            appearance: AppearanceConfig {
                theme: "dark".into(),
                accent_color: "violet".into(),
                reduced_motion: false,
                compact_view: false,
            },
            advanced: AdvancedConfig {
                log_level: "info".into(),
                git_timeout_seconds: 30,
                custom_git_binary: None,
                cache_ttl_minutes: 5,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitoredPath {
    pub id: String,
    pub path: PathBuf,
    pub scope: String,
    pub custom_label: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralConfig {
    pub language: String,
    pub launch_at_login: bool,
    pub minimize_to_tray: bool,
    pub check_app_updates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathsConfig {
    pub monitored: Vec<MonitoredPath>,
    pub default_install_directory: PathBuf,
}

impl PathsConfig {
    pub fn discover_default_paths() -> Vec<MonitoredPath> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        Self::discover_default_paths_from(&home)
    }

    fn discover_default_paths_from(home: &std::path::Path) -> Vec<MonitoredPath> {
        let mut list = Vec::new();
        let mut idx = 1;

        let standard_candidates = vec![
            (
                home.join(".agents/skills"),
                "agents",
                "AI Agents Skills (~/.agents/skills)",
            ),
            (
                home.join(".codex/skills"),
                "codex",
                "OpenAI Codex Skills (~/.codex/skills)",
            ),
            (
                home.join(".claude/skills"),
                "claude",
                "Claude Desktop / Code (~/.claude/skills)",
            ),
            (
                home.join(".gemini/config/skills"),
                "antigravity",
                "Gemini CLI / Antigravity (~/.gemini/config/skills)",
            ),
            (
                home.join(".gemini/antigravity/builtin/skills"),
                "antigravity",
                "Gemini Built-in (~/.gemini/antigravity/builtin/skills)",
            ),
            (
                home.join(".cursor/skills"),
                "cursor",
                "Cursor AI Skills (~/.cursor/skills)",
            ),
            (
                home.join(".config/skills"),
                "global",
                "Global CLI Skills (~/.config/skills)",
            ),
        ];

        for (p, scope, label) in standard_candidates {
            if p.exists() {
                list.push(MonitoredPath {
                    id: format!("p{}", idx),
                    path: p,
                    scope: scope.to_string(),
                    custom_label: Some(label.to_string()),
                    enabled: true,
                });
                idx += 1;
            }
        }

        // Check plugin directories containing skills
        let plugin_roots = vec![
            (home.join(".gemini/config/plugins"), "antigravity"),
            (home.join(".claude/plugins"), "claude"),
            (home.join(".codex/plugins"), "codex"),
        ];

        for (root, scope) in plugin_roots {
            if !root.exists() {
                continue;
            }

            // Plugins are commonly stored as <plugin>/<version>/skills, so a
            // one-level lookup misses installed packages from these agents.
            for entry in walkdir::WalkDir::new(&root)
                .follow_links(false)
                .max_depth(5)
                .into_iter()
                .filter_map(Result::ok)
            {
                let skills_dir = entry.path();
                if !entry.file_type().is_dir() || entry.file_name() != "skills" {
                    continue;
                }
                if list
                    .iter()
                    .any(|known: &MonitoredPath| known.path == skills_dir)
                {
                    continue;
                }

                let name = skills_dir
                    .parent()
                    .and_then(|parent| parent.file_name())
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_else(|| "package".to_string());
                list.push(MonitoredPath {
                    id: format!("p{}", idx),
                    path: skills_dir.to_path_buf(),
                    scope: scope.to_string(),
                    custom_label: Some(format!("Plugin Skills: {}", name)),
                    enabled: true,
                });
                idx += 1;
            }
        }

        if list.is_empty() {
            list.push(MonitoredPath {
                id: "p1".into(),
                path: home.join(".claude/skills"),
                scope: "claude".into(),
                custom_label: None,
                enabled: true,
            });
        }

        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn discovers_codex_and_nested_plugin_skill_directories() {
        let root = std::env::temp_dir().join(format!(
            "skillsync-paths-test-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let codex = root.join(".codex/skills");
        let nested_plugin = root.join(".claude/plugins/cache/example/1.0.0/skills");
        fs::create_dir_all(&codex).unwrap();
        fs::create_dir_all(&nested_plugin).unwrap();

        let paths = PathsConfig::discover_default_paths_from(&root);

        assert!(paths
            .iter()
            .any(|path| path.path == codex && path.scope == "codex"));
        assert!(paths
            .iter()
            .any(|path| path.path == nested_plugin && path.scope == "claude"));

        let _ = fs::remove_dir_all(root);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatesConfig {
    pub auto_check_frequency: String,
    pub auto_install: String,
    pub concurrency_limit: usize,
    pub backup_retention_days: u32,
    pub allow_prerelease: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationsConfig {
    pub enabled: bool,
    pub on_update_found: bool,
    pub on_update_success: bool,
    pub on_update_failure: bool,
    pub sound: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppearanceConfig {
    pub theme: String,
    pub accent_color: String,
    pub reduced_motion: bool,
    pub compact_view: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvancedConfig {
    pub log_level: String,
    pub git_timeout_seconds: u64,
    pub custom_git_binary: Option<PathBuf>,
    pub cache_ttl_minutes: u64,
}

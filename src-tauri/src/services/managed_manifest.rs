use crate::models::skill::ManagedItemType;
use crate::services::manifest::{SkillManifest, SkillManifestKind};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedManifestKind {
    Skill(SkillManifestKind),
    McpConfig,
    LaravelBoost,
    Plugin,
}

/// Validates only explicit formats. Directory names and generic package files
/// are intentionally insufficient: a failed classification must never result
/// in a snapshot or an update of an unrelated application.
pub struct ManagedManifest;

impl ManagedManifest {
    pub fn validate(
        item_type: &ManagedItemType,
        dir: &Path,
    ) -> Result<ManagedManifestKind, String> {
        match item_type {
            ManagedItemType::Skill => SkillManifest::validate(dir).map(ManagedManifestKind::Skill),
            ManagedItemType::Mcp => Self::validate_mcp(dir),
            ManagedItemType::Plugin => Self::validate_plugin(dir),
        }
    }

    pub fn validate_mcp(dir: &Path) -> Result<ManagedManifestKind, String> {
        for filename in ["mcp.json", ".mcp.json"] {
            let path = dir.join(filename);
            if !path.is_file() {
                continue;
            }
            let document = Self::read_json(&path)?;
            if document
                .get("mcpServers")
                .is_some_and(serde_json::Value::is_object)
                || document
                    .get("server")
                    .is_some_and(serde_json::Value::is_object)
            {
                return Ok(ManagedManifestKind::McpConfig);
            }
            return Err(format!(
                "{} nie zawiera obiektu mcpServers ani server",
                filename
            ));
        }

        let composer_path = dir.join("composer.json");
        if composer_path.is_file() {
            let composer = Self::read_json(&composer_path)?;
            let package_name = composer.get("name").and_then(serde_json::Value::as_str);
            let uses_laravel_mcp = composer
                .get("require")
                .and_then(serde_json::Value::as_object)
                .is_some_and(|requires| {
                    requires.contains_key("laravel/mcp") || requires.contains_key("laravel/boost")
                });
            if package_name == Some("laravel/boost") || uses_laravel_mcp {
                return Ok(ManagedManifestKind::LaravelBoost);
            }
        }

        Err("Brak obsługiwanego manifestu MCP: mcp.json, .mcp.json lub jawnej integracji laravel/mcp w composer.json".to_string())
    }

    pub fn validate_plugin(dir: &Path) -> Result<ManagedManifestKind, String> {
        for relative in [
            ".claude-plugin/plugin.json",
            ".codex-plugin/plugin.json",
            ".cursor-plugin/plugin.json",
            "plugin.json",
        ] {
            let path = dir.join(relative);
            if !path.is_file() {
                continue;
            }
            let plugin = Self::read_json(&path)?;
            let name = plugin.get("name").and_then(serde_json::Value::as_str);
            if name.is_none_or(|name| name.trim().is_empty()) {
                return Err(format!("{} nie zawiera nazwy pluginu", relative));
            }
            return Ok(ManagedManifestKind::Plugin);
        }

        Err("Brak obsługiwanego manifestu pluginu (.claude-plugin/plugin.json, .codex-plugin/plugin.json, .cursor-plugin/plugin.json lub plugin.json)".to_string())
    }

    fn read_json(path: &Path) -> Result<serde_json::Value, String> {
        let content = fs::read_to_string(path)
            .map_err(|error| format!("Nie można odczytać {}: {error}", path.display()))?;
        serde_json::from_str(&content)
            .map_err(|_| format!("{} nie zawiera poprawnego JSON", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_dir(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "skillsync-managed-manifest-{name}-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ))
    }

    #[test]
    fn accepts_a_standard_mcp_server_manifest() {
        let dir = fixture_dir("mcp");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("mcp.json"),
            r#"{"mcpServers":{"demo":{"command":"node"}}}"#,
        )
        .unwrap();

        assert_eq!(
            ManagedManifest::validate_mcp(&dir),
            Ok(ManagedManifestKind::McpConfig)
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn accepts_laravel_boost_only_when_its_mcp_dependency_is_explicit() {
        let dir = fixture_dir("boost");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("composer.json"),
            r#"{"name":"laravel/boost","require":{"laravel/mcp":"^1.0"}}"#,
        )
        .unwrap();

        assert_eq!(
            ManagedManifest::validate_mcp(&dir),
            Ok(ManagedManifestKind::LaravelBoost)
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_an_ordinary_composer_project_as_an_mcp_server() {
        let dir = fixture_dir("composer-project");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("composer.json"), r#"{"name":"acme/site"}"#).unwrap();

        assert!(ManagedManifest::validate_mcp(&dir).is_err());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn accepts_the_claude_plugin_manifest_used_by_superpowers() {
        let dir = fixture_dir("superpowers");
        fs::create_dir_all(dir.join(".claude-plugin")).unwrap();
        fs::write(
            dir.join(".claude-plugin/plugin.json"),
            r#"{"name":"superpowers","version":"6.3.0"}"#,
        )
        .unwrap();

        assert_eq!(
            ManagedManifest::validate_plugin(&dir),
            Ok(ManagedManifestKind::Plugin)
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_an_unnamed_plugin_file() {
        let dir = fixture_dir("invalid-plugin");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("plugin.json"), r#"{"version":"1.0.0"}"#).unwrap();

        assert!(ManagedManifest::validate_plugin(&dir).is_err());
        let _ = fs::remove_dir_all(dir);
    }
}

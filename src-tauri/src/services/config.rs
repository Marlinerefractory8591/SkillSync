use crate::errors::SkillSyncError;
use crate::models::config::AppConfig;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

pub struct ConfigService;

impl ConfigService {
    pub fn get_config_path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        let app_dir = base.join("SkillSync");
        let _ = fs::create_dir_all(&app_dir);
        app_dir.join("config.json")
    }

    pub fn load_config() -> AppConfig {
        let path = Self::get_config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut cfg) = serde_json::from_str::<AppConfig>(&content) {
                    let discovered = crate::models::config::PathsConfig::discover_default_paths();
                    let mut changed = false;
                    for disc in discovered {
                        if !cfg.paths.monitored.iter().any(|m| m.path == disc.path) {
                            cfg.paths.monitored.push(disc);
                            changed = true;
                        }
                    }
                    if changed {
                        let _ = Self::save_config(&cfg);
                    }
                    return cfg;
                }
            }
        }
        let default_config = AppConfig::default();
        let _ = Self::save_config(&default_config);
        default_config
    }

    pub fn save_config(config: &AppConfig) -> Result<(), SkillSyncError> {
        let path = Self::get_config_path();
        let tmp_path = path.with_extension("json.tmp");

        let json_str = serde_json::to_string_pretty(config)
            .map_err(|e| SkillSyncError::Config(e.to_string()))?;

        let mut file = File::create(&tmp_path)?;
        file.write_all(json_str.as_bytes())?;
        file.sync_all()?;

        // Atomic replace
        fs::rename(tmp_path, path)?;
        Ok(())
    }
}

use crate::models::config::AppConfig;
use crate::services::config::ConfigService;

#[tauri::command]
pub async fn get_config() -> Result<AppConfig, String> {
    Ok(ConfigService::load_config())
}

#[tauri::command]
pub async fn save_config(config: AppConfig) -> Result<(), String> {
    ConfigService::save_config(&config).map_err(|e| e.to_string())
}

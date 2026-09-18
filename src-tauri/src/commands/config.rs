use crate::models::config::AppConfig;
use crate::services::config::ConfigService;

#[tauri::command]
pub async fn get_config() -> Result<AppConfig, String> {
    Ok(ConfigService::load_config())
}

#[tauri::command]
pub async fn save_config(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    ConfigService::save_config(&config).map_err(|e| e.to_string())?;
    if let Some(tray) = app.tray_by_id("skillsync-tray") {
        tray.set_visible(config.general.show_tray_icon)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

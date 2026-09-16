pub mod commands;
pub mod errors;
pub mod models;
pub mod services;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_app_version,
            check_app_update,
            scan_skills,
            check_github_update,
            update_single_skill,
            checkout_custom_version,
            batch_update_skills,
            rollback_skill,
            get_backups_list,
            open_in_editor,
            open_url,
            get_config,
            save_config
        ])
        .run(tauri::generate_context!())
        .expect("Error while running SkillSync desktop application");
}

pub mod commands;
pub mod errors;
pub mod models;
pub mod services;

use commands::*;
use services::config::ConfigService;
use services::scan_queue::ScanQueue;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ScanQueue::new())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let show = MenuItem::with_id(app, "show", "Show SkillSync", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit SkillSync", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            let tray = TrayIconBuilder::with_id("skillsync-tray")
                .icon(
                    app.default_window_icon()
                        .expect("SkillSync must have a bundled application icon")
                        .clone(),
                )
                .tooltip("SkillSync")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_main_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if matches!(
                        event,
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        }
                    ) {
                        show_main_window(tray.app_handle());
                    }
                })
                .build(app)?;
            tray.set_visible(ConfigService::load_config().general.show_tray_icon)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if ConfigService::load_config().general.minimize_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            scan_skills,
            check_github_update,
            set_branch_override,
            set_repository_override,
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

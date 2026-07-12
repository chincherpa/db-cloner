mod commands;
mod config;
mod jobs;
mod pg;
mod scheduler;
mod secrets;
mod tools;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri_plugin_notification::NotificationExt;

/// Fire-and-forget OS notification (backup finished/failed etc.).
pub fn notify(app: &tauri::AppHandle, body: &str) {
    let _ = app
        .notification()
        .builder()
        .title("DB Cloner")
        .body(body)
        .show();
}

fn setup_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "DB Cloner anzeigen", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Beenden", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let mut builder = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("DB Cloner")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

pub fn run() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("rustls crypto provider");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(jobs::JobRegistry::default())
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::connections::list_connections,
            commands::connections::upsert_connection,
            commands::connections::delete_connection,
            commands::connections::parse_connection_string,
            commands::connections::test_connection,
            commands::browse::get_overview,
            commands::browse::get_table_page,
            commands::backup::start_backup,
            commands::backup::cancel_job,
            commands::backup::list_backups,
            commands::backup::delete_backup,
            commands::restore::start_restore,
            commands::schedule::get_schedule_status,
        ])
        .setup(|app| {
            setup_tray(app.handle())?;
            scheduler::start(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Keep running in the tray so scheduled backups continue.
                let minimize = config::load(&window.app_handle().clone())
                    .map(|c| c.settings.minimize_to_tray)
                    .unwrap_or(false);
                if minimize {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use crate::config::{self, AppSettings};

#[tauri::command]
pub fn get_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    Ok(config::load(&app)?.settings)
}

#[tauri::command]
pub fn save_settings(app: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    let mut cfg = config::load(&app)?;
    cfg.settings = settings;
    config::save(&app, &cfg)
}

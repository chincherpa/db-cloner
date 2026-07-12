use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub backup_dir: Option<String>,
    pub theme: String,
    pub minimize_to_tray: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            backup_dir: None,
            theme: "system".into(),
            minimize_to_tray: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Schedule {
    pub enabled: bool,
    /// "daily" | "weekly" | "interval"
    pub kind: String,
    /// "HH:MM" local time, for daily/weekly
    pub time: String,
    /// 0 = Monday .. 6 = Sunday, for weekly
    pub weekday: u8,
    /// for interval
    pub every_hours: u32,
    /// 0 = keep all
    pub keep_last: u32,
}

impl Default for Schedule {
    fn default() -> Self {
        Self {
            enabled: false,
            kind: "daily".into(),
            time: "03:00".into(),
            weekday: 0,
            every_hours: 12,
            keep_last: 10,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct ConnectionMeta {
    pub id: String,
    pub name: String,
    pub color: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub dbname: String,
    pub schedule: Schedule,
}

impl Default for ConnectionMeta {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            color: "#3ecf8e".into(),
            host: String::new(),
            port: 5432,
            user: "postgres".into(),
            dbname: "postgres".into(),
            schedule: Schedule::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ConfigFile {
    pub settings: AppSettings,
    pub connections: Vec<ConnectionMeta>,
}

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Konfigurationsverzeichnis nicht gefunden: {e}"))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("Konnte {dir:?} nicht anlegen: {e}"))?;
    Ok(dir.join("config.json"))
}

pub fn load(app: &tauri::AppHandle) -> Result<ConfigFile, String> {
    let path = config_path(app)?;
    if !path.exists() {
        return Ok(ConfigFile::default());
    }
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("Konnte Konfiguration nicht lesen: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("Konfiguration ist beschädigt: {e}"))
}

pub fn save(app: &tauri::AppHandle, cfg: &ConfigFile) -> Result<(), String> {
    let path = config_path(app)?;
    let raw = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    std::fs::write(&path, raw).map_err(|e| format!("Konnte Konfiguration nicht speichern: {e}"))
}

pub fn get_connection(app: &tauri::AppHandle, id: &str) -> Result<ConnectionMeta, String> {
    load(app)?
        .connections
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| "Verbindung nicht gefunden".to_string())
}

/// Turns a connection name into a safe directory name.
pub fn sanitize_dir_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.').to_string();
    if trimmed.is_empty() {
        "unnamed".into()
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_replaces_path_separators() {
        assert_eq!(sanitize_dir_name("prod/db:1"), "prod_db_1");
        assert_eq!(sanitize_dir_name("  .hidden.  "), "hidden");
        assert_eq!(sanitize_dir_name("///"), "___");
        assert_eq!(sanitize_dir_name(""), "unnamed");
        assert_eq!(sanitize_dir_name("Mein Projekt"), "Mein Projekt");
    }
}

use serde::Serialize;

use crate::config;
use crate::scheduler;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleStatus {
    pub connection_id: String,
    /// RFC 3339, local offset
    pub last_backup: Option<String>,
    pub next_run: Option<String>,
}

#[tauri::command]
pub fn get_schedule_status(app: tauri::AppHandle) -> Result<Vec<ScheduleStatus>, String> {
    let cfg = config::load(&app)?;
    let now = chrono::Local::now();
    Ok(cfg
        .connections
        .iter()
        .map(|conn| {
            let last = scheduler::last_backup_time(&app, &conn.id);
            let next = scheduler::next_run(&conn.schedule, last, now);
            ScheduleStatus {
                connection_id: conn.id.clone(),
                last_backup: last.map(|d| d.to_rfc3339()),
                next_run: next.map(|d| d.to_rfc3339()),
            }
        })
        .collect())
}

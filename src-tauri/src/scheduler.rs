//! Background scheduler: checks every minute whether a connection's backup
//! schedule is due and kicks off a backup. Missed occurrences (app was
//! closed) are caught up on the next tick after launch.

use chrono::{DateTime, Datelike, Duration, Local, NaiveTime, TimeZone};

use crate::commands::backup;
use crate::config::{self, Schedule};

pub fn start(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        // Give the app a moment to finish startup before the first tick.
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        loop {
            if let Err(e) = tick(&app).await {
                eprintln!("[scheduler] {e}");
            }
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });
}

async fn tick(app: &tauri::AppHandle) -> Result<(), String> {
    let cfg = config::load(app)?;
    if cfg
        .settings
        .backup_dir
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        return Ok(());
    }

    let now = Local::now();
    for conn in cfg.connections.iter().filter(|c| c.schedule.enabled) {
        let last = last_backup_time(app, &conn.id);
        if is_due(&conn.schedule, last, now) {
            // Errors like "already running" or "no password stored" are
            // expected here; the backup task itself reports real failures.
            let _ = backup::start_backup_inner(app.clone(), conn.id.clone(), "scheduled").await;
        }
    }
    Ok(())
}

pub fn last_backup_time(app: &tauri::AppHandle, connection_id: &str) -> Option<DateTime<Local>> {
    let backups = backup::collect_backups(app).ok()?;
    backups
        .iter()
        .filter(|b| b.manifest.connection_id == connection_id)
        .filter_map(|b| DateTime::parse_from_rfc3339(&b.manifest.finished_at).ok())
        .map(|dt| dt.with_timezone(&Local))
        .max()
}

fn parse_time(s: &str) -> NaiveTime {
    NaiveTime::parse_from_str(s, "%H:%M").unwrap_or(NaiveTime::MIN)
}

fn at_local(date: chrono::NaiveDate, time: NaiveTime) -> Option<DateTime<Local>> {
    Local.from_local_datetime(&date.and_time(time)).earliest()
}

/// The most recent scheduled occurrence at or before `now`
/// (daily/weekly only; intervals are relative to the last backup).
pub fn last_occurrence(s: &Schedule, now: DateTime<Local>) -> Option<DateTime<Local>> {
    let time = parse_time(&s.time);
    match s.kind.as_str() {
        "daily" => {
            let today = at_local(now.date_naive(), time)?;
            if today <= now {
                Some(today)
            } else {
                at_local(now.date_naive() - Duration::days(1), time)
            }
        }
        "weekly" => {
            let days_back = (now.weekday().num_days_from_monday() as i64 - s.weekday as i64)
                .rem_euclid(7);
            let candidate = at_local(now.date_naive() - Duration::days(days_back), time)?;
            if candidate <= now {
                Some(candidate)
            } else {
                at_local(candidate.date_naive() - Duration::days(7), time)
            }
        }
        _ => None,
    }
}

pub fn is_due(s: &Schedule, last: Option<DateTime<Local>>, now: DateTime<Local>) -> bool {
    if !s.enabled {
        return false;
    }
    match s.kind.as_str() {
        "interval" => match last {
            None => true,
            Some(l) => now - l >= Duration::hours(s.every_hours.max(1) as i64),
        },
        _ => match last_occurrence(s, now) {
            Some(occurrence) => last.is_none_or(|l| l < occurrence),
            None => false,
        },
    }
}

/// When the next backup will run (for the UI).
pub fn next_run(
    s: &Schedule,
    last: Option<DateTime<Local>>,
    now: DateTime<Local>,
) -> Option<DateTime<Local>> {
    if !s.enabled {
        return None;
    }
    if is_due(s, last, now) {
        return Some(now);
    }
    match s.kind.as_str() {
        "interval" => last.map(|l| l + Duration::hours(s.every_hours.max(1) as i64)),
        "daily" => last_occurrence(s, now).map(|o| o + Duration::days(1)),
        "weekly" => last_occurrence(s, now).map(|o| o + Duration::days(7)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schedule(kind: &str) -> Schedule {
        Schedule {
            enabled: true,
            kind: kind.into(),
            time: "03:00".into(),
            weekday: 0, // Monday
            every_hours: 12,
            keep_last: 5,
        }
    }

    fn local(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<Local> {
        Local.with_ymd_and_hms(y, m, d, h, min, 0).unwrap()
    }

    #[test]
    fn disabled_schedule_is_never_due() {
        let mut s = schedule("daily");
        s.enabled = false;
        assert!(!is_due(&s, None, local(2026, 7, 12, 12, 0)));
    }

    #[test]
    fn interval_due_after_enough_hours() {
        let s = schedule("interval");
        let now = local(2026, 7, 12, 12, 0);
        assert!(is_due(&s, None, now));
        assert!(!is_due(&s, Some(local(2026, 7, 12, 6, 0)), now));
        assert!(is_due(&s, Some(local(2026, 7, 12, 0, 0)), now));
    }

    #[test]
    fn daily_due_once_after_scheduled_time() {
        let s = schedule("daily");
        // Before 03:00 with a backup from yesterday 03:01 → not due.
        assert!(!is_due(
            &s,
            Some(local(2026, 7, 11, 3, 1)),
            local(2026, 7, 12, 2, 0)
        ));
        // After 03:00, last backup was yesterday → due.
        assert!(is_due(
            &s,
            Some(local(2026, 7, 11, 3, 1)),
            local(2026, 7, 12, 4, 0)
        ));
        // Already backed up today after 03:00 → not due.
        assert!(!is_due(
            &s,
            Some(local(2026, 7, 12, 3, 5)),
            local(2026, 7, 12, 12, 0)
        ));
    }

    #[test]
    fn daily_catches_up_missed_run() {
        let s = schedule("daily");
        // App was closed for three days; the most recent occurrence counts.
        assert!(is_due(
            &s,
            Some(local(2026, 7, 9, 3, 1)),
            local(2026, 7, 12, 9, 0)
        ));
    }

    #[test]
    fn weekly_occurrence_math() {
        let s = schedule("weekly"); // Monday 03:00
        // 2026-07-12 is a Sunday → last occurrence is Monday 2026-07-06.
        let occ = last_occurrence(&s, local(2026, 7, 12, 12, 0)).unwrap();
        assert_eq!(occ, local(2026, 7, 6, 3, 0));
        assert!(is_due(&s, Some(local(2026, 7, 5, 3, 0)), local(2026, 7, 12, 12, 0)));
        assert!(!is_due(&s, Some(local(2026, 7, 6, 3, 30)), local(2026, 7, 12, 12, 0)));
    }

    #[test]
    fn next_run_after_todays_backup_is_tomorrow() {
        let s = schedule("daily");
        let next = next_run(&s, Some(local(2026, 7, 12, 3, 5)), local(2026, 7, 12, 12, 0)).unwrap();
        assert_eq!(next, local(2026, 7, 13, 3, 0));
    }
}

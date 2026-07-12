use std::path::PathBuf;
use tauri::Manager;

use crate::commands::backup::{pg_env, run_tool, Manifest};
use crate::config;
use crate::jobs::{emit, JobEvent, JobRegistry};
use crate::{notify, secrets, tools};

#[tauri::command]
pub async fn start_restore(
    app: tauri::AppHandle,
    backup_path: String,
    target_id: String,
) -> Result<String, String> {
    let dir = PathBuf::from(&backup_path);
    let dump = dir.join("db.dump");
    if !dump.is_file() {
        return Err(format!("{} enthält kein db.dump.", dir.display()));
    }
    let manifest: Manifest = std::fs::read_to_string(dir.join("manifest.json"))
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();

    let meta = config::get_connection(&app, &target_id)?;
    let password = secrets::get_password(&target_id)?;

    let registry = app.state::<JobRegistry>();
    if !registry.try_start_backup(&target_id) {
        return Err(format!("Für „{}“ läuft bereits ein Backup/Restore.", meta.name));
    }
    let job_id = uuid::Uuid::new_v4().to_string();
    let job = registry.create(&job_id);

    let app2 = app.clone();
    let job_id2 = job_id.clone();
    tauri::async_runtime::spawn(async move {
        let progress = |percent: f32, phase: &str| {
            emit(
                &app2,
                JobEvent::Progress {
                    job_id: job_id2.clone(),
                    percent,
                    phase: phase.into(),
                },
            );
        };

        let source = if manifest.connection_name.is_empty() {
            dir.display().to_string()
        } else {
            format!("{} ({})", manifest.connection_name, manifest.finished_at)
        };
        progress(-1.0, &format!("Stelle „{source}“ nach „{}“ wieder her…", meta.name));

        let result: Result<u32, String> = async {
            let pg_restore = tools::find_pg_tool(&app2, "pg_restore")?;
            let args: Vec<String> = vec![
                "-h".into(),
                meta.host.clone(),
                "-p".into(),
                meta.port.to_string(),
                "-U".into(),
                meta.user.clone(),
                "-d".into(),
                meta.dbname.clone(),
                "--no-password".into(),
                "--verbose".into(),
                // --clean --if-exists: drop and recreate objects that exist.
                // --no-owner: the Supabase `postgres` role is no superuser and
                // cannot reassign ownership.
                "--clean".into(),
                "--if-exists".into(),
                "--no-owner".into(),
                dump.display().to_string(),
            ];
            let envs = pg_env(&password);

            let mut ignored_errors: u32 = 0;
            let run = run_tool(&job, &pg_restore, &args, &envs, |line| {
                emit(
                    &app2,
                    JobEvent::Log {
                        job_id: job_id2.clone(),
                        line: line.into(),
                    },
                );
                // "errors ignored on restore: N"
                if let Some(rest) = line.split("errors ignored on restore:").nth(1) {
                    ignored_errors = rest.trim().parse().unwrap_or(0);
                }
            })
            .await;

            match run {
                Ok(()) => Ok(0),
                // pg_restore exits with 1 when it finished but ignored errors
                // (expected on managed Postgres: ownership, extension objects).
                Err(_) if ignored_errors > 0 => Ok(ignored_errors),
                Err(e) => Err(format!("pg_restore: {e}")),
            }
        }
        .await;

        let registry = app2.state::<JobRegistry>();
        registry.finish_backup(&meta.id);
        registry.remove(&job_id2);

        match result {
            Ok(ignored) => {
                let msg = if ignored == 0 {
                    format!("Restore nach „{}“ abgeschlossen.", meta.name)
                } else {
                    format!(
                        "Restore nach „{}“ abgeschlossen — {ignored} unkritische Fehler übersprungen (Details im Log).",
                        meta.name
                    )
                };
                notify(&app2, &msg);
                emit(
                    &app2,
                    JobEvent::Done {
                        job_id: job_id2,
                        kind: "restore".into(),
                        message: msg,
                        path: None,
                    },
                );
            }
            Err(e) => {
                let msg = format!("Restore nach „{}“ fehlgeschlagen: {e}", meta.name);
                notify(&app2, &msg);
                emit(
                    &app2,
                    JobEvent::Failed {
                        job_id: job_id2,
                        kind: "restore".into(),
                        message: msg,
                    },
                );
            }
        }
    });

    Ok(job_id)
}

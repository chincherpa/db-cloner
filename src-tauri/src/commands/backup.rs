use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::Manager;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::commands::browse;
use crate::config::{self, sanitize_dir_name, ConnectionMeta};
use crate::jobs::{emit, Job, JobEvent, JobRegistry};
use crate::{notify, pg, secrets, tools};

/// Supabase-managed schemas the `postgres` role cannot fully read or that
/// cannot be restored anyway. A no-op on databases that don't have them.
const EXCLUDED_SCHEMAS: &[&str] = &[
    "realtime",
    "_realtime",
    "pgbouncer",
    "pgsodium",
    "pgsodium_masks",
    "vault",
];

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ManifestTable {
    pub schema: String,
    pub name: String,
    pub rows: i64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Manifest {
    pub app_version: String,
    pub connection_id: String,
    pub connection_name: String,
    pub host: String,
    pub dbname: String,
    pub pg_version: String,
    /// RFC 3339
    pub started_at: String,
    pub finished_at: String,
    pub duration_ms: u64,
    /// "manual" | "scheduled"
    pub trigger: String,
    pub total_rows: i64,
    pub tables: Vec<ManifestTable>,
    /// file name -> size in bytes
    pub files: BTreeMap<String, u64>,
}

#[tauri::command]
pub async fn start_backup(app: tauri::AppHandle, connection_id: String) -> Result<String, String> {
    start_backup_inner(app, connection_id, "manual").await
}

pub async fn start_backup_inner(
    app: tauri::AppHandle,
    connection_id: String,
    trigger: &str,
) -> Result<String, String> {
    let registry = app.state::<JobRegistry>();
    let meta = config::get_connection(&app, &connection_id)?;
    let password = secrets::get_password(&connection_id)?;
    let backup_dir = config::load(&app)?
        .settings
        .backup_dir
        .filter(|d| !d.trim().is_empty())
        .ok_or("Kein Backup-Pfad konfiguriert. Bitte zuerst in den Einstellungen festlegen.")?;

    if !registry.try_start_backup(&connection_id) {
        return Err(format!("Für „{}“ läuft bereits ein Backup.", meta.name));
    }

    let job_id = uuid::Uuid::new_v4().to_string();
    let job = registry.create(&job_id);
    let trigger = trigger.to_string();

    let app2 = app.clone();
    let job_id2 = job_id.clone();
    tauri::async_runtime::spawn(async move {
        let result = run_backup(&app2, &job, &job_id2, &meta, &password, &backup_dir, &trigger).await;
        let registry = app2.state::<JobRegistry>();
        registry.finish_backup(&meta.id);
        registry.remove(&job_id2);
        match result {
            Ok(manifest) => {
                let msg = format!(
                    "Backup von „{}“ abgeschlossen: {} Tabellen, {} Zeilen in {:.1}s.",
                    meta.name,
                    manifest.tables.len(),
                    manifest.total_rows,
                    manifest.duration_ms as f64 / 1000.0
                );
                notify(&app2, &msg);
                emit(
                    &app2,
                    JobEvent::Done {
                        job_id: job_id2,
                        kind: "backup".into(),
                        message: msg,
                        path: None,
                    },
                );
                if trigger == "scheduled" && meta.schedule.keep_last > 0 {
                    if let Err(e) = apply_retention(&app2, &meta) {
                        notify(&app2, &format!("Aufräumen alter Backups fehlgeschlagen: {e}"));
                    }
                }
            }
            Err(e) => {
                let msg = format!("Backup von „{}“ fehlgeschlagen: {e}", meta.name);
                notify(&app2, &msg);
                emit(
                    &app2,
                    JobEvent::Failed {
                        job_id: job_id2,
                        kind: "backup".into(),
                        message: msg,
                    },
                );
            }
        }
    });

    Ok(job_id)
}

async fn run_backup(
    app: &tauri::AppHandle,
    job: &Arc<Job>,
    job_id: &str,
    meta: &ConnectionMeta,
    password: &str,
    backup_dir: &str,
    trigger: &str,
) -> Result<Manifest, String> {
    let started = chrono::Utc::now();
    let started_instant = std::time::Instant::now();

    let progress = |percent: f32, phase: &str| {
        emit(
            app,
            JobEvent::Progress {
                job_id: job_id.into(),
                percent,
                phase: phase.into(),
            },
        );
    };
    let log = |line: &str| {
        emit(
            app,
            JobEvent::Log {
                job_id: job_id.into(),
                line: line.into(),
            },
        );
    };

    progress(2.0, "Verbinde mit Datenbank…");
    let client = pg::connect(meta, password).await?;
    let pg_version: String = client
        .query_one("SELECT version()", &[])
        .await
        .map_err(|e| pg::friendly_error(&e.to_string(), meta))?
        .get(0);
    log(&format!("Server: {pg_version}"));

    progress(5.0, "Zähle Tabellen und Zeilen…");
    let tables = browse::tables_with_counts(&client).await?;
    let manifest_tables: Vec<ManifestTable> = tables
        .iter()
        .map(|(s, t, r)| ManifestTable {
            schema: s.clone(),
            name: t.clone(),
            rows: *r,
        })
        .collect();
    let total_rows: i64 = manifest_tables.iter().map(|t| t.rows.max(0)).sum();
    let total_tables = manifest_tables.len().max(1);
    log(&format!(
        "{} Tabellen, {} Zeilen insgesamt.",
        manifest_tables.len(),
        total_rows
    ));

    let stamp = chrono::Local::now().format("%Y-%m-%d_%H%M%S").to_string();
    let target_dir = PathBuf::from(backup_dir)
        .join(sanitize_dir_name(&meta.name))
        .join(&stamp);
    std::fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Backup-Ordner {target_dir:?} konnte nicht angelegt werden: {e}"))?;
    log(&format!("Backup-Ordner: {}", target_dir.display()));

    let pg_dump = tools::find_pg_tool(app, "pg_dump")?;
    log(&format!("Verwende {}", pg_dump.display()));

    let base_args = |extra: &[String]| -> Vec<String> {
        let mut args = vec![
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
        ];
        for schema in EXCLUDED_SCHEMAS {
            args.push(format!("--exclude-schema={schema}"));
        }
        args.extend_from_slice(extra);
        args
    };
    let envs = pg_env(password);

    // Phase 1: full dump, custom format (compressed, everything pg_dump can
    // see: schema, data, views, functions, triggers, RLS policies, sequences).
    progress(8.0, "Erstelle vollständigen Dump (db.dump)…");
    let dump_path = target_dir.join("db.dump");
    let mut dumped: usize = 0;
    run_tool(
        job,
        &pg_dump,
        &base_args(&[
            "--format=custom".into(),
            "--file".into(),
            dump_path.display().to_string(),
        ]),
        &envs,
        |line| {
            log(line);
            if line.contains("dumping contents of table") {
                dumped += 1;
                let pct = 10.0 + 75.0 * (dumped as f32 / total_tables as f32).min(1.0);
                progress(pct, "Erstelle vollständigen Dump (db.dump)…");
            }
        },
    )
    .await
    .map_err(|e| format!("pg_dump (custom format): {e}"))?;

    // Phase 2: human-readable, diff-able schema-only SQL.
    progress(88.0, "Exportiere Schema als SQL (schema.sql)…");
    let schema_path = target_dir.join("schema.sql");
    run_tool(
        job,
        &pg_dump,
        &base_args(&[
            "--schema-only".into(),
            "--format=plain".into(),
            "--file".into(),
            schema_path.display().to_string(),
        ]),
        &envs,
        |line| log(line),
    )
    .await
    .map_err(|e| format!("pg_dump (schema.sql): {e}"))?;

    progress(96.0, "Schreibe Manifest…");
    let mut files = BTreeMap::new();
    for name in ["db.dump", "schema.sql"] {
        if let Ok(md) = std::fs::metadata(target_dir.join(name)) {
            files.insert(name.to_string(), md.len());
        }
    }

    let finished = chrono::Utc::now();
    let manifest = Manifest {
        app_version: app.package_info().version.to_string(),
        connection_id: meta.id.clone(),
        connection_name: meta.name.clone(),
        host: meta.host.clone(),
        dbname: meta.dbname.clone(),
        pg_version,
        started_at: started.to_rfc3339(),
        finished_at: finished.to_rfc3339(),
        duration_ms: started_instant.elapsed().as_millis() as u64,
        trigger: trigger.into(),
        total_rows,
        tables: manifest_tables,
        files,
    };
    let manifest_json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    std::fs::write(target_dir.join("manifest.json"), manifest_json)
        .map_err(|e| format!("Manifest konnte nicht geschrieben werden: {e}"))?;

    progress(100.0, "Fertig");
    Ok(manifest)
}

pub fn pg_env(password: &str) -> Vec<(String, String)> {
    vec![
        ("PGPASSWORD".into(), password.into()),
        // "prefer": TLS wherever the server offers it (Supabase always does),
        // plaintext fallback for local/dev servers without TLS.
        ("PGSSLMODE".into(), "prefer".into()),
        ("PGCONNECT_TIMEOUT".into(), "15".into()),
    ]
}

/// Runs an external tool, streaming its stderr lines (pg_dump/pg_restore log
/// there) into `on_line`. Supports cancellation via the job handle.
pub async fn run_tool(
    job: &Arc<Job>,
    program: &Path,
    args: &[String],
    envs: &[(String, String)],
    mut on_line: impl FnMut(&str),
) -> Result<(), String> {
    let mut cmd = tokio::process::Command::new(program);
    cmd.args(args)
        .envs(envs.iter().cloned())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("{} konnte nicht gestartet werden: {e}", program.display()))?;

    let stderr = child.stderr.take().expect("stderr piped");
    let stdout = child.stdout.take().expect("stdout piped");
    tauri::async_runtime::spawn(async move {
        // Drain stdout so the child never blocks on a full pipe.
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(_)) = lines.next_line().await {}
    });

    *job.child.lock().await = Some(child);

    let mut lines = BufReader::new(stderr).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        on_line(&line);
    }

    let status = {
        let mut guard = job.child.lock().await;
        let status = match guard.as_mut() {
            Some(child) => child.wait().await.map_err(|e| e.to_string())?,
            None => return Err("Abgebrochen.".into()),
        };
        *guard = None;
        status
    };

    if job.cancelled.load(Ordering::Relaxed) {
        return Err("Abgebrochen.".into());
    }
    if !status.success() {
        return Err(format!("Prozess beendet mit {status}"));
    }
    Ok(())
}

#[tauri::command]
pub async fn cancel_job(
    registry: tauri::State<'_, JobRegistry>,
    job_id: String,
) -> Result<(), String> {
    if let Some(job) = registry.get(&job_id) {
        job.cancelled.store(true, Ordering::Relaxed);
        if let Some(child) = job.child.lock().await.as_mut() {
            let _ = child.kill().await;
        }
    }
    Ok(())
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BackupEntry {
    pub path: String,
    pub size_bytes: u64,
    pub manifest: Manifest,
}

pub fn collect_backups(app: &tauri::AppHandle) -> Result<Vec<BackupEntry>, String> {
    let Some(backup_dir) = config::load(app)?.settings.backup_dir else {
        return Ok(vec![]);
    };
    let root = PathBuf::from(&backup_dir);
    if !root.is_dir() {
        return Ok(vec![]);
    }

    let mut entries = vec![];
    for conn_dir in std::fs::read_dir(&root).map_err(|e| e.to_string())?.flatten() {
        if !conn_dir.path().is_dir() {
            continue;
        }
        let Ok(sub) = std::fs::read_dir(conn_dir.path()) else {
            continue;
        };
        for backup in sub.flatten() {
            let dir = backup.path();
            let manifest_path = dir.join("manifest.json");
            let Ok(raw) = std::fs::read_to_string(&manifest_path) else {
                continue;
            };
            let Ok(manifest) = serde_json::from_str::<Manifest>(&raw) else {
                continue;
            };
            let size_bytes = std::fs::read_dir(&dir)
                .map(|rd| {
                    rd.flatten()
                        .filter_map(|f| f.metadata().ok())
                        .map(|md| md.len())
                        .sum()
                })
                .unwrap_or(0);
            entries.push(BackupEntry {
                path: dir.display().to_string(),
                size_bytes,
                manifest,
            });
        }
    }
    entries.sort_by(|a, b| b.manifest.finished_at.cmp(&a.manifest.finished_at));
    Ok(entries)
}

#[tauri::command]
pub fn list_backups(app: tauri::AppHandle) -> Result<Vec<BackupEntry>, String> {
    collect_backups(&app)
}

#[tauri::command]
pub fn delete_backup(app: tauri::AppHandle, path: String) -> Result<(), String> {
    let backup_dir = config::load(&app)?
        .settings
        .backup_dir
        .ok_or("Kein Backup-Pfad konfiguriert.")?;
    let root = PathBuf::from(&backup_dir)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let target = PathBuf::from(&path).canonicalize().map_err(|e| e.to_string())?;

    // Only ever delete directories that are inside the backup root and that
    // actually look like one of our backups.
    if !target.starts_with(&root) || target == root {
        return Err("Pfad liegt außerhalb des Backup-Ordners.".into());
    }
    if !target.join("manifest.json").is_file() {
        return Err("Dieser Ordner ist kein DB-Cloner-Backup (manifest.json fehlt).".into());
    }
    std::fs::remove_dir_all(&target).map_err(|e| format!("Löschen fehlgeschlagen: {e}"))
}

/// Deletes the oldest scheduled backups beyond `keep_last`.
fn apply_retention(app: &tauri::AppHandle, meta: &ConnectionMeta) -> Result<(), String> {
    let keep = meta.schedule.keep_last as usize;
    if keep == 0 {
        return Ok(());
    }
    let backups: Vec<BackupEntry> = collect_backups(app)?
        .into_iter()
        .filter(|b| b.manifest.connection_id == meta.id)
        .collect(); // already sorted newest first
    for old in backups.iter().skip(keep) {
        let _ = std::fs::remove_dir_all(&old.path);
    }
    Ok(())
}

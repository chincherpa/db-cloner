use serde::Serialize;
use std::time::Instant;

use crate::config::{self, ConnectionMeta};
use crate::{pg, secrets};

#[tauri::command]
pub fn list_connections(app: tauri::AppHandle) -> Result<Vec<ConnectionMeta>, String> {
    Ok(config::load(&app)?.connections)
}

#[tauri::command]
pub fn upsert_connection(
    app: tauri::AppHandle,
    mut meta: ConnectionMeta,
    password: Option<String>,
) -> Result<ConnectionMeta, String> {
    if meta.name.trim().is_empty() {
        return Err("Bitte einen Namen für die Verbindung angeben.".into());
    }
    if meta.host.trim().is_empty() {
        return Err("Bitte einen Host angeben.".into());
    }
    if meta.id.is_empty() {
        meta.id = uuid::Uuid::new_v4().to_string();
    }

    if let Some(pw) = password.as_deref() {
        if !pw.is_empty() {
            secrets::set_password(&meta.id, pw)?;
        }
    }

    let mut cfg = config::load(&app)?;
    if let Some(existing) = cfg.connections.iter_mut().find(|c| c.id == meta.id) {
        *existing = meta.clone();
    } else {
        cfg.connections.push(meta.clone());
    }
    config::save(&app, &cfg)?;
    Ok(meta)
}

#[tauri::command]
pub fn delete_connection(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let mut cfg = config::load(&app)?;
    cfg.connections.retain(|c| c.id != id);
    config::save(&app, &cfg)?;
    secrets::delete_password(&id);
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedConnection {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub dbname: String,
    pub password: Option<String>,
}

/// Parses a libpq connection string (URI or key-value form), e.g. the one
/// copied from the Supabase dashboard ("Connect" → Session pooler).
#[tauri::command]
pub fn parse_connection_string(value: String) -> Result<ParsedConnection, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("Connection-String ist leer.".into());
    }
    // Query parameters like ?pgbouncer=true are irrelevant here (TLS is
    // always enforced) and would trip the parser — strip them.
    let cleaned = match (
        trimmed.starts_with("postgres://") || trimmed.starts_with("postgresql://"),
        trimmed.find('?'),
    ) {
        (true, Some(idx)) => &trimmed[..idx],
        _ => trimmed,
    };

    let cfg: tokio_postgres::Config = cleaned
        .parse()
        .map_err(|e| format!("Connection-String konnte nicht gelesen werden: {e}"))?;

    let host = cfg
        .get_hosts()
        .iter()
        .find_map(|h| match h {
            tokio_postgres::config::Host::Tcp(s) => Some(s.clone()),
            #[allow(unreachable_patterns)]
            _ => None,
        })
        .ok_or("Kein Host im Connection-String gefunden.")?;

    let port = cfg.get_ports().first().copied().unwrap_or(5432);
    let user = cfg.get_user().unwrap_or("postgres").to_string();
    let dbname = cfg.get_dbname().unwrap_or("postgres").to_string();
    let password = cfg
        .get_password()
        .map(|p| String::from_utf8_lossy(p).to_string())
        .filter(|p| !p.is_empty() && p != "[YOUR-PASSWORD]");

    Ok(ParsedConnection {
        host,
        port,
        user,
        dbname,
        password,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    pub version: String,
    pub latency_ms: u64,
}

/// Tests a connection. `password` overrides the stored one (used while
/// editing, before anything is saved).
#[tauri::command]
pub async fn test_connection(
    meta: ConnectionMeta,
    password: Option<String>,
) -> Result<TestResult, String> {
    let pw = match password.filter(|p| !p.is_empty()) {
        Some(p) => p,
        None => secrets::get_password(&meta.id)?,
    };
    let started = Instant::now();
    let client = pg::connect(&meta, &pw).await?;
    let row = client
        .query_one("SELECT version()", &[])
        .await
        .map_err(|e| pg::friendly_error(&e.to_string(), &meta))?;
    Ok(TestResult {
        version: row.get::<_, String>(0),
        latency_ms: started.elapsed().as_millis() as u64,
    })
}

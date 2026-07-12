//! Locates the PostgreSQL client tools (pg_dump, pg_restore).
//!
//! Search order:
//! 1. bundled app resources (`pg-tools/bin/`, populated by scripts/fetch-pg-tools)
//! 2. the system PATH

use std::path::PathBuf;
use tauri::Manager;

pub fn find_pg_tool(app: &tauri::AppHandle, tool: &str) -> Result<PathBuf, String> {
    let exe = if cfg!(windows) {
        format!("{tool}.exe")
    } else {
        tool.to_string()
    };

    if let Ok(res_dir) = app.path().resource_dir() {
        for candidate in [
            res_dir.join("pg-tools").join("bin").join(&exe),
            res_dir.join("pg-tools").join(&exe),
        ] {
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    // Dev fallback: the repo-local pg-tools dir next to the manifest.
    #[cfg(debug_assertions)]
    {
        let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("pg-tools")
            .join("bin")
            .join(&exe);
        if dev.is_file() {
            return Ok(dev);
        }
    }

    which(&exe).ok_or_else(|| {
        format!(
            "'{tool}' wurde nicht gefunden — weder als gebündelte Resource noch im PATH. \
             Führe `scripts/fetch-pg-tools` aus oder installiere die PostgreSQL Client Tools."
        )
    })
}

fn which(exe: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(exe);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

//! Long-running jobs (backup/restore) with live events to the frontend.

use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::Emitter;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum JobEvent {
    #[serde(rename_all = "camelCase")]
    Log { job_id: String, line: String },
    #[serde(rename_all = "camelCase")]
    Progress {
        job_id: String,
        /// 0..=100, or -1 for indeterminate
        percent: f32,
        phase: String,
    },
    #[serde(rename_all = "camelCase")]
    Done {
        job_id: String,
        kind: String,
        message: String,
        path: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Failed {
        job_id: String,
        kind: String,
        message: String,
    },
}

pub fn emit(app: &tauri::AppHandle, event: JobEvent) {
    let _ = app.emit("job-event", event);
}

pub struct Job {
    pub cancelled: Arc<AtomicBool>,
    pub child: Arc<tokio::sync::Mutex<Option<tokio::process::Child>>>,
}

impl Job {
    fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            child: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }
}

#[derive(Default)]
pub struct JobRegistry {
    jobs: Mutex<HashMap<String, Arc<Job>>>,
    /// Connection ids with a backup currently running (prevents double runs).
    running_backups: Mutex<HashSet<String>>,
}

impl JobRegistry {
    pub fn create(&self, job_id: &str) -> Arc<Job> {
        let job = Arc::new(Job::new());
        self.jobs
            .lock()
            .unwrap()
            .insert(job_id.to_string(), job.clone());
        job
    }

    pub fn remove(&self, job_id: &str) {
        self.jobs.lock().unwrap().remove(job_id);
    }

    pub fn get(&self, job_id: &str) -> Option<Arc<Job>> {
        self.jobs.lock().unwrap().get(job_id).cloned()
    }

    /// Returns false if a backup for this connection is already running.
    pub fn try_start_backup(&self, connection_id: &str) -> bool {
        self.running_backups
            .lock()
            .unwrap()
            .insert(connection_id.to_string())
    }

    pub fn finish_backup(&self, connection_id: &str) {
        self.running_backups.lock().unwrap().remove(connection_id);
    }
}

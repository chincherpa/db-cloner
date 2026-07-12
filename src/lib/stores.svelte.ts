import {
  api,
  type AppSettings,
  type BackupEntry,
  type ConnectionMeta,
  type JobEvent,
  type Overview,
  type ScheduleStatus,
} from "./api";

export type View = "welcome" | "database" | "history" | "settings";

export interface Toast {
  id: number;
  kind: "success" | "error" | "info";
  message: string;
}

export interface JobState {
  jobId: string;
  kind: string;
  phase: string;
  percent: number;
  lines: string[];
  status: "running" | "done" | "failed";
  message: string;
}

export const app = $state({
  view: "welcome" as View,
  settings: null as AppSettings | null,
  connections: [] as ConnectionMeta[],
  scheduleStatus: {} as Record<string, ScheduleStatus>,
  activeId: null as string | null,
  overview: null as Overview | null,
  overviewLoading: false,
  overviewError: null as string | null,
  selectedTable: null as { schema: string; table: string } | null,
  backups: [] as BackupEntry[],
  toasts: [] as Toast[],
  jobs: [] as JobState[],
  editConnection: null as ConnectionMeta | null,
  showConnectionModal: false,
  restoreBackup: null as BackupEntry | null,
});

let toastSeq = 0;

export function toast(kind: Toast["kind"], message: string) {
  const id = ++toastSeq;
  app.toasts.push({ id, kind, message });
  setTimeout(() => {
    app.toasts = app.toasts.filter((t) => t.id !== id);
  }, kind === "error" ? 8000 : 4500);
}

export async function loadConnections() {
  app.connections = await api.listConnections();
}

export async function loadSettings() {
  app.settings = await api.getSettings();
}

export async function loadScheduleStatus() {
  try {
    const list = await api.getScheduleStatus();
    app.scheduleStatus = Object.fromEntries(list.map((s) => [s.connectionId, s]));
  } catch {
    // non-fatal
  }
}

export async function loadBackups() {
  try {
    app.backups = await api.listBackups();
  } catch (e) {
    toast("error", String(e));
  }
}

export async function openDatabase(id: string) {
  app.activeId = id;
  app.view = "database";
  app.selectedTable = null;
  app.overview = null;
  app.overviewError = null;
  app.overviewLoading = true;
  try {
    app.overview = await api.getOverview(id);
  } catch (e) {
    app.overviewError = String(e);
  } finally {
    app.overviewLoading = false;
  }
}

export function activeConnection(): ConnectionMeta | null {
  return app.connections.find((c) => c.id === app.activeId) ?? null;
}

export function handleJobEvent(ev: JobEvent) {
  let job = app.jobs.find((j) => j.jobId === ev.jobId);
  if (!job) {
    job = {
      jobId: ev.jobId,
      kind: "backup",
      phase: "",
      percent: -1,
      lines: [],
      status: "running",
      message: "",
    };
    app.jobs.push(job);
  }
  switch (ev.type) {
    case "log":
      job.lines.push(ev.line);
      if (job.lines.length > 2000) job.lines.splice(0, job.lines.length - 2000);
      break;
    case "progress":
      job.percent = ev.percent;
      job.phase = ev.phase;
      break;
    case "done":
      job.kind = ev.kind;
      job.status = "done";
      job.percent = 100;
      job.message = ev.message;
      toast("success", ev.message);
      void loadBackups();
      void loadScheduleStatus();
      break;
    case "failed":
      job.kind = ev.kind;
      job.status = "failed";
      job.message = ev.message;
      toast("error", ev.message);
      break;
  }
}

export async function startBackup(id: string) {
  try {
    await api.startBackup(id);
    toast("info", "Backup gestartet…");
  } catch (e) {
    toast("error", String(e));
  }
}

export function dismissJob(jobId: string) {
  app.jobs = app.jobs.filter((j) => j.jobId !== jobId);
}

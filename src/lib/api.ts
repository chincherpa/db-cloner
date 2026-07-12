import { invoke } from "@tauri-apps/api/core";

export interface Schedule {
  enabled: boolean;
  kind: "daily" | "weekly" | "interval";
  time: string;
  weekday: number;
  everyHours: number;
  keepLast: number;
}

export interface ConnectionMeta {
  id: string;
  name: string;
  color: string;
  host: string;
  port: number;
  user: string;
  dbname: string;
  schedule: Schedule;
}

export interface AppSettings {
  backupDir: string | null;
  theme: "system" | "light" | "dark";
  minimizeToTray: boolean;
}

export interface ParsedConnection {
  host: string;
  port: number;
  user: string;
  dbname: string;
  password: string | null;
}

export interface TestResult {
  version: string;
  latencyMs: number;
}

export interface TableInfo {
  name: string;
  rows: number;
}

export interface PolicyInfo {
  table: string;
  name: string;
  cmd: string;
}

export interface SchemaInfo {
  name: string;
  tables: TableInfo[];
  views: string[];
  functions: string[];
  policies: PolicyInfo[];
}

export interface Overview {
  pgVersion: string;
  schemas: SchemaInfo[];
  totalTables: number;
  totalRows: number;
}

export interface ColumnInfo {
  name: string;
  dataType: string;
}

export interface TablePage {
  columns: ColumnInfo[];
  rows: (string | null)[][];
  totalRows: number;
}

export interface ManifestTable {
  schema: string;
  name: string;
  rows: number;
}

export interface Manifest {
  appVersion: string;
  connectionId: string;
  connectionName: string;
  host: string;
  dbname: string;
  pgVersion: string;
  startedAt: string;
  finishedAt: string;
  durationMs: number;
  trigger: string;
  totalRows: number;
  tables: ManifestTable[];
  files: Record<string, number>;
}

export interface BackupEntry {
  path: string;
  sizeBytes: number;
  manifest: Manifest;
}

export interface ScheduleStatus {
  connectionId: string;
  lastBackup: string | null;
  nextRun: string | null;
}

export type JobEvent =
  | { type: "log"; jobId: string; line: string }
  | { type: "progress"; jobId: string; percent: number; phase: string }
  | { type: "done"; jobId: string; kind: string; message: string; path: string | null }
  | { type: "failed"; jobId: string; kind: string; message: string };

export const api = {
  getSettings: () => invoke<AppSettings>("get_settings"),
  saveSettings: (settings: AppSettings) => invoke<void>("save_settings", { settings }),

  listConnections: () => invoke<ConnectionMeta[]>("list_connections"),
  upsertConnection: (meta: ConnectionMeta, password: string | null) =>
    invoke<ConnectionMeta>("upsert_connection", { meta, password }),
  deleteConnection: (id: string) => invoke<void>("delete_connection", { id }),
  parseConnectionString: (value: string) =>
    invoke<ParsedConnection>("parse_connection_string", { value }),
  testConnection: (meta: ConnectionMeta, password: string | null) =>
    invoke<TestResult>("test_connection", { meta, password }),

  getOverview: (id: string) => invoke<Overview>("get_overview", { id }),
  getTablePage: (args: {
    id: string;
    schema: string;
    table: string;
    limit: number;
    offset: number;
    sortColumn: string | null;
    sortDesc: boolean;
  }) => invoke<TablePage>("get_table_page", { ...args }),

  startBackup: (connectionId: string) => invoke<string>("start_backup", { connectionId }),
  cancelJob: (jobId: string) => invoke<void>("cancel_job", { jobId }),
  listBackups: () => invoke<BackupEntry[]>("list_backups"),
  deleteBackup: (path: string) => invoke<void>("delete_backup", { path }),
  startRestore: (backupPath: string, targetId: string) =>
    invoke<string>("start_restore", { backupPath, targetId }),

  getScheduleStatus: () => invoke<ScheduleStatus[]>("get_schedule_status"),
};

export function emptyConnection(): ConnectionMeta {
  return {
    id: "",
    name: "",
    color: "#3ecf8e",
    host: "",
    port: 5432,
    user: "postgres",
    dbname: "postgres",
    schedule: {
      enabled: false,
      kind: "daily",
      time: "03:00",
      weekday: 0,
      everyHours: 12,
      keepLast: 10,
    },
  };
}

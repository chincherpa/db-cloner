export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let value = bytes;
  let unit = "B";
  for (const u of units) {
    if (value < 1024) break;
    value /= 1024;
    unit = u;
  }
  return `${value.toFixed(value >= 100 ? 0 : 1)} ${unit}`;
}

export function formatNumber(n: number): string {
  return new Intl.NumberFormat("de-DE").format(n);
}

export function formatDate(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleString("de-DE", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function timeAgo(iso: string): string {
  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) return "";
  const secs = Math.floor((Date.now() - then) / 1000);
  if (secs < 0) return "gleich";
  if (secs < 60) return "gerade eben";
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `vor ${mins} min`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `vor ${hours} h`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `vor ${days} d`;
  return formatDate(iso);
}

export function timeUntil(iso: string): string {
  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) return "";
  const secs = Math.floor((then - Date.now()) / 1000);
  if (secs <= 60) return "gleich";
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `in ${mins} min`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `in ${hours} h`;
  return `in ${Math.floor(hours / 24)} d`;
}

export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms} ms`;
  const secs = ms / 1000;
  if (secs < 60) return `${secs.toFixed(1)} s`;
  const mins = Math.floor(secs / 60);
  return `${mins}:${String(Math.round(secs % 60)).padStart(2, "0")} min`;
}

<script lang="ts">
  import { ask } from "@tauri-apps/plugin-dialog";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { api, type BackupEntry } from "../api";
  import { app, loadBackups, toast } from "../stores.svelte";
  import { formatBytes, formatDate, formatDuration, formatNumber } from "../format";

  const groups = $derived.by(() => {
    const map = new Map<string, BackupEntry[]>();
    for (const backup of app.backups) {
      const key = backup.manifest.connectionName || "Unbekannt";
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(backup);
    }
    return [...map.entries()];
  });

  async function remove(backup: BackupEntry) {
    const yes = await ask(
      `Backup von „${backup.manifest.connectionName}“ vom ${formatDate(backup.manifest.finishedAt)} unwiderruflich löschen?`,
      { title: "Backup löschen", kind: "warning" }
    );
    if (!yes) return;
    try {
      await api.deleteBackup(backup.path);
      toast("success", "Backup gelöscht.");
      await loadBackups();
    } catch (e) {
      toast("error", String(e));
    }
  }

  async function reveal(backup: BackupEntry) {
    try {
      await revealItemInDir(backup.path);
    } catch (e) {
      toast("error", String(e));
    }
  }
</script>

<header class="border-b border-slate-200 bg-white px-5 py-3.5 dark:border-slate-800 dark:bg-panel">
  <h1 class="text-base font-semibold tracking-tight">Backup-Historie</h1>
  <p class="text-xs text-slate-500 dark:text-slate-400">
    Alle lokalen Backups — wiederherstellen, im Dateimanager öffnen oder löschen.
  </p>
</header>

<div class="flex-1 overflow-y-auto p-5">
  {#if app.backups.length === 0}
    <div class="mx-auto mt-16 max-w-sm text-center text-sm text-slate-500 dark:text-slate-400">
      Noch keine Backups vorhanden. Verbinde dich mit einer Datenbank und klicke auf
      <span class="font-medium text-accent">Backup to Disk</span>.
    </div>
  {/if}

  {#each groups as [name, backups] (name)}
    <section class="mb-6">
      <h2 class="mb-2 flex items-center gap-2 text-sm font-semibold">
        {name}
        <span class="text-xs font-normal text-slate-400">
          {backups.length} {backups.length === 1 ? "Backup" : "Backups"} ·
          {formatBytes(backups.reduce((n, b) => n + b.sizeBytes, 0))}
        </span>
      </h2>
      <div class="overflow-hidden rounded-xl border border-slate-200 dark:border-slate-800">
        {#each backups as backup, i (backup.path)}
          <div
            class="flex flex-wrap items-center gap-x-4 gap-y-1 bg-white px-4 py-3 dark:bg-panel
                   {i > 0 ? 'border-t border-slate-100 dark:border-slate-800' : ''}"
          >
            <div class="min-w-44">
              <div class="text-sm font-medium tabular-nums">
                {formatDate(backup.manifest.finishedAt)}
              </div>
              <div class="text-[11px] text-slate-400 selectable" title={backup.path}>
                {backup.manifest.dbname}@{backup.manifest.host}
              </div>
            </div>
            <span
              class="rounded-full px-2 py-0.5 text-[11px] font-medium
                     {backup.manifest.trigger === 'scheduled'
                ? 'bg-sky-100 text-sky-700 dark:bg-sky-900/40 dark:text-sky-300'
                : 'bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-300'}"
            >
              {backup.manifest.trigger === "scheduled" ? "geplant" : "manuell"}
            </span>
            <span class="text-xs tabular-nums text-slate-500 dark:text-slate-400">
              {formatBytes(backup.sizeBytes)}
            </span>
            <span class="text-xs tabular-nums text-slate-500 dark:text-slate-400">
              {backup.manifest.tables.length} Tabellen · {formatNumber(backup.manifest.totalRows)} Zeilen
            </span>
            <span class="text-xs tabular-nums text-slate-400">
              {formatDuration(backup.manifest.durationMs)}
            </span>
            <div class="ml-auto flex items-center gap-1.5">
              <button
                class="rounded-lg border border-slate-300 px-2.5 py-1.5 text-xs font-medium transition hover:border-accent hover:text-accent dark:border-slate-700"
                onclick={() => (app.restoreBackup = backup)}
              >
                Wiederherstellen
              </button>
              <button
                class="rounded-lg border border-slate-300 px-2.5 py-1.5 text-xs transition hover:bg-slate-50 dark:border-slate-700 dark:hover:bg-slate-800"
                onclick={() => void reveal(backup)}
                title="Im Dateimanager anzeigen"
              >
                Ordner
              </button>
              <button
                class="rounded-lg border border-slate-300 px-2.5 py-1.5 text-xs text-red-600 transition hover:border-red-400 hover:bg-red-50 dark:border-slate-700 dark:text-red-400 dark:hover:bg-red-950/30"
                onclick={() => void remove(backup)}
              >
                Löschen
              </button>
            </div>
          </div>
        {/each}
      </div>
    </section>
  {/each}
</div>

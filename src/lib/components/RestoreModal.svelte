<script lang="ts">
  import { api } from "../api";
  import { app, toast } from "../stores.svelte";
  import { formatBytes, formatDate, formatNumber } from "../format";

  const backup = $derived(app.restoreBackup);

  let targetId = $state("");
  let confirmation = $state("");
  let starting = $state(false);

  $effect(() => {
    if (backup && !targetId) {
      const original = app.connections.find((c) => c.id === backup.manifest.connectionId);
      targetId = original?.id ?? app.connections[0]?.id ?? "";
    }
  });

  const target = $derived(app.connections.find((c) => c.id === targetId) ?? null);
  const confirmed = $derived(target !== null && confirmation.trim() === target.name);

  function close() {
    app.restoreBackup = null;
  }

  async function start() {
    if (!backup || !target || !confirmed) return;
    starting = true;
    try {
      await api.startRestore(backup.path, target.id);
      toast("info", `Restore nach „${target.name}“ gestartet…`);
      close();
    } catch (e) {
      toast("error", String(e));
    } finally {
      starting = false;
    }
  }
</script>

{#if backup}
  <div
    class="fixed inset-0 z-40 flex items-center justify-center bg-black/50 p-4 backdrop-blur-sm"
    role="presentation"
    onclick={(e) => e.target === e.currentTarget && close()}
  >
    <div
      class="w-full max-w-lg rounded-2xl border border-slate-200 bg-white shadow-2xl dark:border-slate-700 dark:bg-panel"
      role="dialog"
      aria-modal="true"
    >
      <div class="border-b border-slate-200 px-5 py-4 dark:border-slate-800">
        <h2 class="text-base font-semibold tracking-tight">Backup wiederherstellen</h2>
        <p class="mt-0.5 text-xs text-slate-500 dark:text-slate-400">
          {backup.manifest.connectionName} · {formatDate(backup.manifest.finishedAt)} ·
          {formatBytes(backup.sizeBytes)} · {backup.manifest.tables.length} Tabellen ·
          {formatNumber(backup.manifest.totalRows)} Zeilen
        </p>
      </div>

      <div class="space-y-4 p-5">
        <label class="block">
          <span class="mb-1 block text-xs font-medium text-slate-500 dark:text-slate-400">Ziel-Datenbank</span>
          <select
            class="w-full rounded-lg border border-slate-300 bg-transparent px-2.5 py-2 text-sm dark:border-slate-600 dark:bg-panel"
            bind:value={targetId}
          >
            {#each app.connections as conn (conn.id)}
              <option value={conn.id}>{conn.name} — {conn.host}/{conn.dbname}</option>
            {/each}
          </select>
        </label>

        <div class="rounded-lg border border-red-300 bg-red-50 p-3 text-xs leading-relaxed text-red-700 dark:border-red-900 dark:bg-red-950/40 dark:text-red-300">
          <b>Achtung:</b> Die Wiederherstellung löscht bestehende Objekte in der Ziel-Datenbank und
          ersetzt sie durch den Stand des Backups (<code>pg_restore --clean</code>). Das kann nicht
          rückgängig gemacht werden.
        </div>

        {#if target}
          <label class="block">
            <span class="mb-1 block text-xs font-medium text-slate-500 dark:text-slate-400">
              Zum Bestätigen den Namen der Ziel-Verbindung eintippen:
              <b class="selectable">{target.name}</b>
            </span>
            <input
              type="text"
              class="w-full rounded-lg border border-slate-300 bg-transparent px-2.5 py-2 font-mono text-sm dark:border-slate-600"
              placeholder={target.name}
              bind:value={confirmation}
            />
          </label>
        {/if}
      </div>

      <div class="flex items-center justify-end gap-2 border-t border-slate-200 px-5 py-4 dark:border-slate-800">
        <button
          class="rounded-lg px-3 py-2 text-sm text-slate-500 transition hover:bg-slate-100 dark:hover:bg-slate-800"
          onclick={close}
        >
          Abbrechen
        </button>
        <button
          class="rounded-lg bg-red-600 px-4 py-2 text-sm font-semibold text-white transition hover:bg-red-700 disabled:cursor-not-allowed disabled:opacity-40"
          disabled={!confirmed || starting}
          onclick={() => void start()}
        >
          Wiederherstellen
        </button>
      </div>
    </div>
  </div>
{/if}

<script lang="ts">
  import { ask, open } from "@tauri-apps/plugin-dialog";
  import { api, emptyConnection, type ConnectionMeta } from "../api";
  import { app, loadConnections, toast } from "../stores.svelte";

  async function save() {
    if (!app.settings) return;
    try {
      await api.saveSettings($state.snapshot(app.settings));
    } catch (e) {
      toast("error", String(e));
    }
  }

  async function pickBackupDir() {
    const dir = await open({ directory: true, title: "Backup-Ablagepfad wählen" });
    if (typeof dir === "string" && app.settings) {
      app.settings.backupDir = dir;
      await save();
      toast("success", "Backup-Pfad gespeichert.");
    }
  }

  function edit(conn: ConnectionMeta) {
    app.editConnection = $state.snapshot(conn);
    app.showConnectionModal = true;
  }

  async function remove(conn: ConnectionMeta) {
    const yes = await ask(
      `Verbindung „${conn.name}“ löschen? Bereits erstellte Backups bleiben erhalten.`,
      { title: "Verbindung löschen", kind: "warning" }
    );
    if (!yes) return;
    try {
      await api.deleteConnection(conn.id);
      if (app.activeId === conn.id) {
        app.activeId = null;
        app.view = "settings";
      }
      await loadConnections();
      toast("success", "Verbindung gelöscht.");
    } catch (e) {
      toast("error", String(e));
    }
  }
</script>

<header class="border-b border-slate-200 bg-white px-5 py-3.5 dark:border-slate-800 dark:bg-panel">
  <h1 class="text-base font-semibold tracking-tight">Einstellungen</h1>
  <p class="text-xs text-slate-500 dark:text-slate-400">
    Credentials deiner Datenbanken, Backup-Ablage und App-Verhalten.
  </p>
</header>

<div class="flex-1 space-y-8 overflow-y-auto p-6">
  {#if app.settings}
    <section class="max-w-2xl">
      <h2 class="mb-3 text-sm font-semibold uppercase tracking-wider text-slate-400">Backups</h2>
      <div class="rounded-xl border border-slate-200 bg-white p-4 dark:border-slate-800 dark:bg-panel">
        <div class="flex items-center gap-3">
          <div class="min-w-0 flex-1">
            <div class="text-sm font-medium">Ablagepfad</div>
            <div class="truncate text-xs text-slate-500 dark:text-slate-400 selectable">
              {app.settings.backupDir ?? "— noch nicht festgelegt —"}
            </div>
          </div>
          <button
            class="rounded-lg bg-accent px-3 py-2 text-sm font-semibold text-ink transition hover:bg-accent-dark hover:text-white"
            onclick={() => void pickBackupDir()}
          >
            Ordner wählen…
          </button>
        </div>
        <p class="mt-3 text-xs leading-relaxed text-slate-400">
          Backups werden als <code>&lt;Pfad&gt;/&lt;Verbindung&gt;/&lt;Zeitstempel&gt;/</code> abgelegt und
          enthalten <code>db.dump</code> (vollständig, komprimiert), <code>schema.sql</code> (lesbar)
          und <code>manifest.json</code>.
        </p>
      </div>
    </section>

    <section class="max-w-2xl">
      <h2 class="mb-3 text-sm font-semibold uppercase tracking-wider text-slate-400">App</h2>
      <div class="space-y-4 rounded-xl border border-slate-200 bg-white p-4 dark:border-slate-800 dark:bg-panel">
        <label class="flex items-center justify-between gap-4">
          <span>
            <span class="block text-sm font-medium">Design</span>
            <span class="block text-xs text-slate-500 dark:text-slate-400">Hell, dunkel oder wie das System.</span>
          </span>
          <select
            class="rounded-lg border border-slate-300 bg-transparent px-2 py-1.5 text-sm dark:border-slate-700 dark:bg-panel"
            bind:value={app.settings.theme}
            onchange={() => void save()}
          >
            <option value="system">System</option>
            <option value="light">Hell</option>
            <option value="dark">Dunkel</option>
          </select>
        </label>
        <label class="flex items-center justify-between gap-4">
          <span>
            <span class="block text-sm font-medium">Beim Schließen ins Tray minimieren</span>
            <span class="block text-xs text-slate-500 dark:text-slate-400">
              Nötig, damit zeitgesteuerte Backups weiterlaufen, wenn das Fenster zu ist.
            </span>
          </span>
          <input
            type="checkbox"
            class="h-5 w-5 accent-accent"
            bind:checked={app.settings.minimizeToTray}
            onchange={() => void save()}
          />
        </label>
      </div>
    </section>
  {/if}

  <section class="max-w-2xl">
    <div class="mb-3 flex items-center justify-between">
      <h2 class="text-sm font-semibold uppercase tracking-wider text-slate-400">Verbindungen</h2>
      <button
        class="rounded-lg border border-slate-300 px-3 py-1.5 text-sm font-medium transition hover:border-accent hover:text-accent dark:border-slate-700"
        onclick={() => {
          app.editConnection = emptyConnection();
          app.showConnectionModal = true;
        }}
      >
        + Neue Verbindung
      </button>
    </div>
    <div class="overflow-hidden rounded-xl border border-slate-200 dark:border-slate-800">
      {#if app.connections.length === 0}
        <div class="bg-white p-6 text-center text-sm text-slate-400 dark:bg-panel">
          Noch keine Verbindungen angelegt.
        </div>
      {/if}
      {#each app.connections as conn, i (conn.id)}
        <div
          class="flex items-center gap-3 bg-white px-4 py-3 dark:bg-panel
                 {i > 0 ? 'border-t border-slate-100 dark:border-slate-800' : ''}"
        >
          <span class="h-2.5 w-2.5 rounded-full" style="background-color: {conn.color}"></span>
          <div class="min-w-0 flex-1">
            <div class="truncate text-sm font-medium">{conn.name}</div>
            <div class="truncate text-xs text-slate-500 dark:text-slate-400 selectable">
              {conn.user}@{conn.host}:{conn.port}/{conn.dbname}
            </div>
          </div>
          {#if conn.schedule.enabled}
            <span class="rounded-full bg-sky-100 px-2 py-0.5 text-[11px] font-medium text-sky-700 dark:bg-sky-900/40 dark:text-sky-300">
              {conn.schedule.kind === "daily"
                ? `täglich ${conn.schedule.time}`
                : conn.schedule.kind === "weekly"
                  ? `wöchentlich ${conn.schedule.time}`
                  : `alle ${conn.schedule.everyHours} h`}
            </span>
          {/if}
          <button
            class="rounded-lg border border-slate-300 px-2.5 py-1.5 text-xs font-medium transition hover:bg-slate-50 dark:border-slate-700 dark:hover:bg-slate-800"
            onclick={() => edit(conn)}
          >
            Bearbeiten
          </button>
          <button
            class="rounded-lg border border-slate-300 px-2.5 py-1.5 text-xs text-red-600 transition hover:border-red-400 hover:bg-red-50 dark:border-slate-700 dark:text-red-400 dark:hover:bg-red-950/30"
            onclick={() => void remove(conn)}
          >
            Löschen
          </button>
        </div>
      {/each}
    </div>
    <p class="mt-3 text-xs leading-relaxed text-slate-400">
      Passwörter werden sicher im Schlüsselbund deines Betriebssystems gespeichert, nie im Klartext auf der Festplatte.
    </p>
  </section>
</div>

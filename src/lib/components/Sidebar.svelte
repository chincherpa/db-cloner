<script lang="ts">
  import { app, openDatabase } from "../stores.svelte";
  import { emptyConnection } from "../api";
  import { timeAgo } from "../format";

  function addConnection() {
    app.editConnection = emptyConnection();
    app.showConnectionModal = true;
  }

  function freshness(id: string): string | null {
    const status = app.scheduleStatus[id];
    return status?.lastBackup ? timeAgo(status.lastBackup) : null;
  }
</script>

<aside
  class="flex w-64 shrink-0 flex-col border-r border-slate-200 bg-white dark:border-slate-800 dark:bg-panel"
>
  <div class="flex items-center gap-2.5 px-4 py-4">
    <div
      class="flex h-8 w-8 items-center justify-center rounded-lg bg-accent/15 text-accent"
    >
      <svg viewBox="0 0 24 24" class="h-5 w-5" fill="none" stroke="currentColor" stroke-width="2">
        <ellipse cx="12" cy="5" rx="8" ry="3" />
        <path d="M4 5v14c0 1.66 3.58 3 8 3s8-1.34 8-3V5" />
        <path d="M4 12c0 1.66 3.58 3 8 3s8-1.34 8-3" />
      </svg>
    </div>
    <div>
      <div class="text-sm font-semibold tracking-tight">DB Cloner</div>
      <div class="text-[11px] text-slate-500 dark:text-slate-400">Supabase Backups</div>
    </div>
  </div>

  <div class="px-3 pb-1 pt-2 text-[11px] font-semibold uppercase tracking-wider text-slate-400">
    Datenbanken
  </div>

  <nav class="flex-1 space-y-0.5 overflow-y-auto px-2">
    {#each app.connections as conn (conn.id)}
      <button
        class="group flex w-full items-center gap-2.5 rounded-lg px-2.5 py-2 text-left transition
               {app.view === 'database' && app.activeId === conn.id
          ? 'bg-slate-100 dark:bg-slate-800'
          : 'hover:bg-slate-50 dark:hover:bg-slate-800/60'}"
        onclick={() => void openDatabase(conn.id)}
      >
        <span
          class="h-2.5 w-2.5 shrink-0 rounded-full"
          style="background-color: {conn.color}"
        ></span>
        <span class="min-w-0 flex-1">
          <span class="block truncate text-sm font-medium">{conn.name}</span>
          <span class="block truncate text-[11px] text-slate-500 dark:text-slate-400">
            {#if freshness(conn.id)}
              Backup {freshness(conn.id)}
            {:else}
              noch kein Backup
            {/if}
          </span>
        </span>
        {#if conn.schedule.enabled}
          <svg
            viewBox="0 0 24 24"
            class="h-3.5 w-3.5 shrink-0 text-slate-400"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <circle cx="12" cy="12" r="9" />
            <path d="M12 7v5l3 3" />
          </svg>
        {/if}
      </button>
    {/each}

    {#if app.connections.length === 0}
      <p class="px-2.5 py-2 text-xs text-slate-400">
        Noch keine Verbindungen. Lege unten die erste an.
      </p>
    {/if}

    <button
      class="mt-1 flex w-full items-center gap-2 rounded-lg border border-dashed border-slate-300 px-2.5
             py-2 text-sm text-slate-500 transition hover:border-accent hover:text-accent
             dark:border-slate-700 dark:text-slate-400"
      onclick={addConnection}
    >
      <span class="text-base leading-none">+</span> Neue Verbindung
    </button>
  </nav>

  <div class="space-y-0.5 border-t border-slate-200 p-2 dark:border-slate-800">
    <button
      class="flex w-full items-center gap-2.5 rounded-lg px-2.5 py-2 text-sm transition
             {app.view === 'history'
        ? 'bg-slate-100 dark:bg-slate-800'
        : 'hover:bg-slate-50 dark:hover:bg-slate-800/60'}"
      onclick={() => (app.view = "history")}
    >
      <svg viewBox="0 0 24 24" class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M3 12a9 9 0 1 0 3-6.7L3 8" />
        <path d="M3 3v5h5" />
        <path d="M12 7v5l4 2" />
      </svg>
      Backup-Historie
      {#if app.backups.length > 0}
        <span class="ml-auto rounded-full bg-slate-200 px-1.5 text-[11px] dark:bg-slate-700">
          {app.backups.length}
        </span>
      {/if}
    </button>
    <button
      class="flex w-full items-center gap-2.5 rounded-lg px-2.5 py-2 text-sm transition
             {app.view === 'settings'
        ? 'bg-slate-100 dark:bg-slate-800'
        : 'hover:bg-slate-50 dark:hover:bg-slate-800/60'}"
      onclick={() => (app.view = "settings")}
    >
      <svg viewBox="0 0 24 24" class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3" />
        <path
          d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h.01a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51h.01a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.01a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
        />
      </svg>
      Einstellungen
    </button>
  </div>
</aside>

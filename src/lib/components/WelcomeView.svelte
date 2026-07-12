<script lang="ts">
  import { app } from "../stores.svelte";
  import { emptyConnection } from "../api";
</script>

<div class="flex flex-1 items-center justify-center p-8">
  <div class="max-w-md text-center">
    <div
      class="mx-auto mb-5 flex h-16 w-16 items-center justify-center rounded-2xl bg-accent/15 text-accent"
    >
      <svg viewBox="0 0 24 24" class="h-9 w-9" fill="none" stroke="currentColor" stroke-width="1.7">
        <ellipse cx="12" cy="5" rx="8" ry="3" />
        <path d="M4 5v14c0 1.66 3.58 3 8 3s8-1.34 8-3V5" />
        <path d="M4 12c0 1.66 3.58 3 8 3s8-1.34 8-3" />
      </svg>
    </div>
    <h1 class="text-xl font-semibold tracking-tight">Willkommen bei DB Cloner</h1>
    <p class="mt-2 text-sm leading-relaxed text-slate-500 dark:text-slate-400">
      Sichere deine Supabase-Datenbanken vollständig auf deine Festplatte — Schema, Daten,
      Views, Funktionen, Trigger und RLS-Policies. Verbinde dich, schau in deine Daten und
      klicke auf <span class="font-medium text-accent">Backup to Disk</span>.
    </p>
    <div class="mt-6 flex items-center justify-center gap-3">
      <button
        class="rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-ink transition hover:bg-accent-dark hover:text-white"
        onclick={() => {
          app.editConnection = emptyConnection();
          app.showConnectionModal = true;
        }}
      >
        Erste Verbindung anlegen
      </button>
      <button
        class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium transition hover:bg-slate-50 dark:border-slate-700 dark:hover:bg-slate-800"
        onclick={() => (app.view = "settings")}
      >
        Einstellungen
      </button>
    </div>
    {#if !app.settings?.backupDir}
      <p class="mt-5 text-xs text-amber-600 dark:text-amber-400">
        Tipp: Lege zuerst in den Einstellungen den Backup-Ablagepfad fest.
      </p>
    {/if}
  </div>
</div>

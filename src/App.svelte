<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import type { JobEvent } from "./lib/api";
  import {
    app,
    handleJobEvent,
    loadBackups,
    loadConnections,
    loadScheduleStatus,
    loadSettings,
    startBackup,
  } from "./lib/stores.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import WelcomeView from "./lib/components/WelcomeView.svelte";
  import DatabaseView from "./lib/components/DatabaseView.svelte";
  import HistoryView from "./lib/components/HistoryView.svelte";
  import SettingsView from "./lib/components/SettingsView.svelte";
  import ConnectionModal from "./lib/components/ConnectionModal.svelte";
  import RestoreModal from "./lib/components/RestoreModal.svelte";
  import JobPanel from "./lib/components/JobPanel.svelte";
  import Toasts from "./lib/components/Toasts.svelte";

  onMount(() => {
    void loadSettings();
    void loadConnections();
    void loadScheduleStatus();
    void loadBackups();
    const unlisten = listen<JobEvent>("job-event", (e) => handleJobEvent(e.payload));
    const interval = setInterval(() => void loadScheduleStatus(), 60_000);
    return () => {
      void unlisten.then((u) => u());
      clearInterval(interval);
    };
  });

  $effect(() => {
    const theme = app.settings?.theme ?? "system";
    const dark =
      theme === "dark" ||
      (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
    document.documentElement.classList.toggle("dark", dark);
  });

  function onKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "b") {
      e.preventDefault();
      if (app.view === "database" && app.activeId) void startBackup(app.activeId);
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="flex h-full bg-slate-100 text-slate-900 dark:bg-ink dark:text-slate-100">
  <Sidebar />
  <main class="flex min-w-0 flex-1 flex-col">
    {#if app.view === "database" && app.activeId}
      <DatabaseView />
    {:else if app.view === "history"}
      <HistoryView />
    {:else if app.view === "settings"}
      <SettingsView />
    {:else}
      <WelcomeView />
    {/if}
  </main>
</div>

<JobPanel />
<Toasts />

{#if app.showConnectionModal}
  <ConnectionModal />
{/if}
{#if app.restoreBackup}
  <RestoreModal />
{/if}

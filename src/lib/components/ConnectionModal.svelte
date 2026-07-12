<script lang="ts">
  import { api, emptyConnection, type TestResult } from "../api";
  import { app, loadConnections, loadScheduleStatus, toast } from "../stores.svelte";

  const WEEKDAYS = ["Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag", "Sonntag"];

  let meta = $state(app.editConnection ? { ...app.editConnection, schedule: { ...app.editConnection.schedule } } : emptyConnection());
  const isNew = meta.id === "";

  let password = $state("");
  let connectionString = $state("");
  let testing = $state(false);
  let saving = $state(false);
  let testResult = $state<TestResult | null>(null);
  let testError = $state<string | null>(null);

  const isDirectSupabase = $derived(
    meta.host.startsWith("db.") && meta.host.endsWith(".supabase.co")
  );

  function close() {
    app.showConnectionModal = false;
    app.editConnection = null;
  }

  async function applyConnectionString() {
    if (!connectionString.trim()) return;
    try {
      const parsed = await api.parseConnectionString(connectionString);
      meta.host = parsed.host;
      meta.port = parsed.port;
      meta.user = parsed.user;
      meta.dbname = parsed.dbname;
      if (parsed.password) password = parsed.password;
      if (!meta.name) {
        // e.g. user "postgres.abcdefgh" on the pooler → project ref as name
        meta.name = parsed.user.includes(".") ? (parsed.user.split(".")[1] ?? parsed.dbname) : parsed.dbname;
      }
      testResult = null;
      testError = null;
      toast("success", "Connection-String übernommen.");
    } catch (e) {
      toast("error", String(e));
    }
  }

  async function test() {
    testing = true;
    testResult = null;
    testError = null;
    try {
      testResult = await api.testConnection($state.snapshot(meta), password || null);
    } catch (e) {
      testError = String(e);
    } finally {
      testing = false;
    }
  }

  async function save() {
    if (isNew && !password) {
      toast("error", "Bitte ein Passwort eingeben.");
      return;
    }
    saving = true;
    try {
      await api.upsertConnection($state.snapshot(meta), password || null);
      await loadConnections();
      await loadScheduleStatus();
      toast("success", `Verbindung „${meta.name}“ gespeichert.`);
      close();
    } catch (e) {
      toast("error", String(e));
    } finally {
      saving = false;
    }
  }
</script>

<div
  class="fixed inset-0 z-40 flex items-center justify-center bg-black/50 p-4 backdrop-blur-sm"
  role="presentation"
  onclick={(e) => e.target === e.currentTarget && close()}
>
  <div
    class="max-h-[90vh] w-full max-w-xl overflow-y-auto rounded-2xl border border-slate-200 bg-white shadow-2xl dark:border-slate-700 dark:bg-panel"
    role="dialog"
    aria-modal="true"
  >
    <div class="flex items-center justify-between border-b border-slate-200 px-5 py-4 dark:border-slate-800">
      <h2 class="text-base font-semibold tracking-tight">
        {isNew ? "Neue Verbindung" : `„${meta.name}“ bearbeiten`}
      </h2>
      <button class="rounded-md px-2 py-1 text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800" onclick={close}>
        ✕
      </button>
    </div>

    <div class="space-y-5 p-5">
      <!-- paste connection string -->
      <div class="rounded-xl border border-slate-200 bg-slate-50 p-3 dark:border-slate-700 dark:bg-panel-2">
        <label class="mb-1.5 block text-xs font-medium text-slate-500 dark:text-slate-400" for="conn-string">
          Connection-String einfügen (Supabase-Dashboard → Connect → <b>Session pooler</b>)
        </label>
        <div class="flex gap-2">
          <input
            id="conn-string"
            type="text"
            class="min-w-0 flex-1 rounded-lg border border-slate-300 bg-white px-2.5 py-1.5 font-mono text-xs selectable dark:border-slate-600 dark:bg-panel"
            placeholder="postgresql://postgres.xxxx:[PASSWORT]@aws-0-eu-central-1.pooler.supabase.com:5432/postgres"
            bind:value={connectionString}
          />
          <button
            class="shrink-0 rounded-lg border border-slate-300 px-3 py-1.5 text-xs font-medium transition hover:border-accent hover:text-accent dark:border-slate-600"
            onclick={() => void applyConnectionString()}
          >
            Übernehmen
          </button>
        </div>
      </div>

      <div class="grid grid-cols-2 gap-4">
        <label class="col-span-2 block sm:col-span-1">
          <span class="mb-1 block text-xs font-medium text-slate-500 dark:text-slate-400">Name</span>
          <input
            type="text"
            class="w-full rounded-lg border border-slate-300 bg-transparent px-2.5 py-1.5 text-sm selectable dark:border-slate-600"
            placeholder="Mein Projekt"
            bind:value={meta.name}
          />
        </label>
        <label class="col-span-2 block sm:col-span-1">
          <span class="mb-1 block text-xs font-medium text-slate-500 dark:text-slate-400">Farbe</span>
          <input type="color" class="h-8 w-full cursor-pointer rounded-lg border border-slate-300 dark:border-slate-600" bind:value={meta.color} />
        </label>
        <label class="col-span-2 block">
          <span class="mb-1 block text-xs font-medium text-slate-500 dark:text-slate-400">Host</span>
          <input
            type="text"
            class="w-full rounded-lg border border-slate-300 bg-transparent px-2.5 py-1.5 font-mono text-sm selectable dark:border-slate-600"
            placeholder="aws-0-eu-central-1.pooler.supabase.com"
            bind:value={meta.host}
          />
        </label>
        <label class="block">
          <span class="mb-1 block text-xs font-medium text-slate-500 dark:text-slate-400">Port</span>
          <input
            type="number"
            min="1"
            max="65535"
            class="w-full rounded-lg border border-slate-300 bg-transparent px-2.5 py-1.5 font-mono text-sm dark:border-slate-600"
            bind:value={meta.port}
          />
        </label>
        <label class="block">
          <span class="mb-1 block text-xs font-medium text-slate-500 dark:text-slate-400">Datenbank</span>
          <input
            type="text"
            class="w-full rounded-lg border border-slate-300 bg-transparent px-2.5 py-1.5 font-mono text-sm selectable dark:border-slate-600"
            bind:value={meta.dbname}
          />
        </label>
        <label class="block">
          <span class="mb-1 block text-xs font-medium text-slate-500 dark:text-slate-400">Benutzer</span>
          <input
            type="text"
            class="w-full rounded-lg border border-slate-300 bg-transparent px-2.5 py-1.5 font-mono text-sm selectable dark:border-slate-600"
            placeholder="postgres.projektref"
            bind:value={meta.user}
          />
        </label>
        <label class="block">
          <span class="mb-1 block text-xs font-medium text-slate-500 dark:text-slate-400">
            Passwort {#if !isNew}<span class="font-normal">(leer = unverändert)</span>{/if}
          </span>
          <input
            type="password"
            class="w-full rounded-lg border border-slate-300 bg-transparent px-2.5 py-1.5 text-sm dark:border-slate-600"
            bind:value={password}
          />
        </label>
      </div>

      {#if isDirectSupabase}
        <div class="rounded-lg border border-amber-300 bg-amber-50 p-3 text-xs leading-relaxed text-amber-800 dark:border-amber-800 dark:bg-amber-950/40 dark:text-amber-300">
          ⚠ <b>db.*.supabase.co</b> ist die Direktverbindung und nur über IPv6 erreichbar. Falls die
          Verbindung fehlschlägt, nutze den <b>Session pooler</b> (Host <i>*.pooler.supabase.com</i>, Port 5432).
        </div>
      {/if}

      <!-- schedule -->
      <div class="rounded-xl border border-slate-200 p-4 dark:border-slate-700">
        <label class="flex items-center justify-between">
          <span>
            <span class="block text-sm font-medium">Zeitgesteuertes Backup</span>
            <span class="block text-xs text-slate-500 dark:text-slate-400">
              Läuft automatisch, solange die App (auch im Tray) geöffnet ist.
            </span>
          </span>
          <input type="checkbox" class="h-5 w-5 accent-accent" bind:checked={meta.schedule.enabled} />
        </label>

        {#if meta.schedule.enabled}
          <div class="mt-3 grid grid-cols-2 gap-3">
            <label class="block">
              <span class="mb-1 block text-xs font-medium text-slate-500 dark:text-slate-400">Rhythmus</span>
              <select
                class="w-full rounded-lg border border-slate-300 bg-transparent px-2 py-1.5 text-sm dark:border-slate-600 dark:bg-panel"
                bind:value={meta.schedule.kind}
              >
                <option value="daily">Täglich</option>
                <option value="weekly">Wöchentlich</option>
                <option value="interval">Alle N Stunden</option>
              </select>
            </label>
            {#if meta.schedule.kind === "interval"}
              <label class="block">
                <span class="mb-1 block text-xs font-medium text-slate-500 dark:text-slate-400">Stunden</span>
                <input
                  type="number"
                  min="1"
                  max="168"
                  class="w-full rounded-lg border border-slate-300 bg-transparent px-2.5 py-1.5 text-sm dark:border-slate-600"
                  bind:value={meta.schedule.everyHours}
                />
              </label>
            {:else}
              <label class="block">
                <span class="mb-1 block text-xs font-medium text-slate-500 dark:text-slate-400">Uhrzeit</span>
                <input
                  type="time"
                  class="w-full rounded-lg border border-slate-300 bg-transparent px-2.5 py-1.5 text-sm dark:border-slate-600"
                  bind:value={meta.schedule.time}
                />
              </label>
            {/if}
            {#if meta.schedule.kind === "weekly"}
              <label class="block">
                <span class="mb-1 block text-xs font-medium text-slate-500 dark:text-slate-400">Wochentag</span>
                <select
                  class="w-full rounded-lg border border-slate-300 bg-transparent px-2 py-1.5 text-sm dark:border-slate-600 dark:bg-panel"
                  bind:value={meta.schedule.weekday}
                >
                  {#each WEEKDAYS as day, i (i)}
                    <option value={i}>{day}</option>
                  {/each}
                </select>
              </label>
            {/if}
            <label class="block">
              <span class="mb-1 block text-xs font-medium text-slate-500 dark:text-slate-400">
                Aufbewahrung (letzte N behalten, 0 = alle)
              </span>
              <input
                type="number"
                min="0"
                max="1000"
                class="w-full rounded-lg border border-slate-300 bg-transparent px-2.5 py-1.5 text-sm dark:border-slate-600"
                bind:value={meta.schedule.keepLast}
              />
            </label>
          </div>
        {/if}
      </div>

      {#if testResult}
        <div class="rounded-lg border border-emerald-300 bg-emerald-50 p-3 text-xs text-emerald-800 dark:border-emerald-800 dark:bg-emerald-950/40 dark:text-emerald-300 selectable">
          ✓ Verbindung OK ({testResult.latencyMs} ms) — {testResult.version.split(" on ")[0]}
        </div>
      {/if}
      {#if testError}
        <div class="rounded-lg border border-red-300 bg-red-50 p-3 text-xs leading-relaxed text-red-700 dark:border-red-900 dark:bg-red-950/40 dark:text-red-300 selectable">
          {testError}
        </div>
      {/if}
    </div>

    <div class="flex items-center justify-between gap-2 border-t border-slate-200 px-5 py-4 dark:border-slate-800">
      <button
        class="flex items-center gap-2 rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium transition hover:bg-slate-50 disabled:opacity-50 dark:border-slate-600 dark:hover:bg-slate-800"
        onclick={() => void test()}
        disabled={testing || !meta.host}
      >
        {#if testing}
          <span class="h-3.5 w-3.5 animate-spin rounded-full border-2 border-accent border-t-transparent"></span>
        {/if}
        Verbindung testen
      </button>
      <div class="flex gap-2">
        <button
          class="rounded-lg px-3 py-2 text-sm text-slate-500 transition hover:bg-slate-100 dark:hover:bg-slate-800"
          onclick={close}
        >
          Abbrechen
        </button>
        <button
          class="rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-ink transition hover:bg-accent-dark hover:text-white disabled:opacity-50"
          onclick={() => void save()}
          disabled={saving || !meta.name || !meta.host}
        >
          Speichern
        </button>
      </div>
    </div>
  </div>
</div>

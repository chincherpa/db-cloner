<script lang="ts">
  import { activeConnection, app, openDatabase, startBackup } from "../stores.svelte";
  import { formatNumber } from "../format";
  import TableGrid from "./TableGrid.svelte";

  const conn = $derived(activeConnection());
  const running = $derived(app.jobs.some((j) => j.status === "running"));

  let openSchemas = $state<Record<string, boolean>>({});

  $effect(() => {
    // Open "public" (or the first schema) once the overview arrives.
    if (app.overview) {
      const names = app.overview.schemas.map((s) => s.name);
      if (!names.some((n) => openSchemas[n])) {
        openSchemas[names.includes("public") ? "public" : (names[0] ?? "")] = true;
      }
    }
  });

  function selectTable(schema: string, table: string) {
    app.selectedTable = { schema, table };
  }
</script>

{#if conn}
  <header
    class="flex items-center gap-3 border-b border-slate-200 bg-white px-5 py-3 dark:border-slate-800 dark:bg-panel"
  >
    <span class="h-3 w-3 rounded-full" style="background-color: {conn.color}"></span>
    <div class="min-w-0">
      <h1 class="truncate text-base font-semibold tracking-tight">{conn.name}</h1>
      <p class="truncate text-xs text-slate-500 dark:text-slate-400 selectable">
        {conn.user}@{conn.host}:{conn.port}/{conn.dbname}
        {#if app.overview}
          · {app.overview.pgVersion.split(" on ")[0]}
        {/if}
      </p>
    </div>
    <div class="ml-auto flex items-center gap-2">
      <button
        class="rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium transition hover:bg-slate-50
               disabled:opacity-50 dark:border-slate-700 dark:hover:bg-slate-800"
        onclick={() => conn && void openDatabase(conn.id)}
        disabled={app.overviewLoading}
        title="Neu laden"
      >
        <svg viewBox="0 0 24 24" class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 12a9 9 0 1 1-3-6.7L21 8" />
          <path d="M21 3v5h-5" />
        </svg>
      </button>
      <button
        class="flex items-center gap-2 rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-ink
               transition hover:bg-accent-dark hover:text-white disabled:cursor-not-allowed disabled:opacity-50"
        onclick={() => conn && void startBackup(conn.id)}
        disabled={running}
        title="Strg+B"
      >
        <svg viewBox="0 0 24 24" class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="2.2">
          <path d="M12 3v12" />
          <path d="m7 10 5 5 5-5" />
          <path d="M4 19h16" />
        </svg>
        Backup to Disk
      </button>
    </div>
  </header>

  <div class="flex min-h-0 flex-1">
    <!-- schema tree -->
    <div
      class="w-72 shrink-0 overflow-y-auto border-r border-slate-200 bg-white/60 p-2 dark:border-slate-800 dark:bg-panel/60"
    >
      {#if app.overviewLoading}
        <div class="space-y-2 p-2">
          {#each Array(6) as _}
            <div class="h-8 animate-pulse rounded-lg bg-slate-200 dark:bg-slate-800"></div>
          {/each}
        </div>
      {:else if app.overviewError}
        <div class="m-2 rounded-lg border border-red-300 bg-red-50 p-3 text-xs leading-relaxed text-red-700 dark:border-red-900 dark:bg-red-950/40 dark:text-red-300 selectable">
          {app.overviewError}
        </div>
      {:else if app.overview}
        {#each app.overview.schemas as schema (schema.name)}
          {@const count =
            schema.tables.length + schema.views.length + schema.functions.length + schema.policies.length}
          {#if count > 0}
            <button
              class="flex w-full items-center gap-1.5 rounded-lg px-2 py-1.5 text-left text-sm font-medium hover:bg-slate-100 dark:hover:bg-slate-800"
              onclick={() => (openSchemas[schema.name] = !openSchemas[schema.name])}
            >
              <svg
                viewBox="0 0 24 24"
                class="h-3.5 w-3.5 text-slate-400 transition-transform {openSchemas[schema.name] ? 'rotate-90' : ''}"
                fill="none"
                stroke="currentColor"
                stroke-width="2.5"
              >
                <path d="m9 6 6 6-6 6" />
              </svg>
              {schema.name}
              <span class="ml-auto text-[11px] font-normal text-slate-400">
                {schema.tables.length} Tab.
              </span>
            </button>
            {#if openSchemas[schema.name]}
              <div class="mb-1 ml-3 border-l border-slate-200 pl-2 dark:border-slate-800">
                {#each schema.tables as table (table.name)}
                  <button
                    class="flex w-full items-center gap-2 rounded-md px-2 py-1 text-left text-[13px] transition
                           {app.selectedTable?.schema === schema.name && app.selectedTable?.table === table.name
                      ? 'bg-accent/15 text-accent-dark dark:text-accent'
                      : 'text-slate-600 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-800'}"
                    onclick={() => selectTable(schema.name, table.name)}
                  >
                    <svg viewBox="0 0 24 24" class="h-3.5 w-3.5 shrink-0 opacity-60" fill="none" stroke="currentColor" stroke-width="2">
                      <rect x="3" y="4" width="18" height="16" rx="2" />
                      <path d="M3 10h18M9 10v10" />
                    </svg>
                    <span class="min-w-0 flex-1 truncate">{table.name}</span>
                    <span class="text-[11px] tabular-nums text-slate-400">
                      {table.rows >= 0 ? formatNumber(table.rows) : "–"}
                    </span>
                  </button>
                {/each}
                {#if schema.views.length > 0}
                  <div class="mt-1 px-2 text-[10px] font-semibold uppercase tracking-wider text-slate-400">
                    Views ({schema.views.length})
                  </div>
                  {#each schema.views as view (view)}
                    <div class="truncate px-2 py-0.5 text-[12px] text-slate-500 dark:text-slate-400">{view}</div>
                  {/each}
                {/if}
                {#if schema.functions.length > 0}
                  <details class="mt-1">
                    <summary class="cursor-pointer px-2 text-[10px] font-semibold uppercase tracking-wider text-slate-400">
                      Funktionen ({schema.functions.length})
                    </summary>
                    {#each schema.functions as fn (fn)}
                      <div class="truncate px-2 py-0.5 text-[12px] text-slate-500 dark:text-slate-400">{fn}()</div>
                    {/each}
                  </details>
                {/if}
                {#if schema.policies.length > 0}
                  <details class="mt-1">
                    <summary class="cursor-pointer px-2 text-[10px] font-semibold uppercase tracking-wider text-slate-400">
                      RLS-Policies ({schema.policies.length})
                    </summary>
                    {#each schema.policies as policy (policy.table + policy.name)}
                      <div class="truncate px-2 py-0.5 text-[12px] text-slate-500 dark:text-slate-400" title="{policy.table}: {policy.cmd}">
                        {policy.table} · {policy.name}
                      </div>
                    {/each}
                  </details>
                {/if}
              </div>
            {/if}
          {/if}
        {/each}
      {/if}
    </div>

    <!-- content -->
    <div class="flex min-w-0 flex-1 flex-col">
      {#if app.selectedTable}
        <TableGrid />
      {:else if app.overview}
        <div class="grid gap-4 p-6 sm:grid-cols-2 lg:grid-cols-4">
          {#each [
            { label: "Schemas", value: app.overview.schemas.filter((s) => s.tables.length + s.views.length + s.functions.length > 0).length },
            { label: "Tabellen", value: app.overview.totalTables },
            { label: "Zeilen gesamt", value: app.overview.totalRows },
            { label: "RLS-Policies", value: app.overview.schemas.reduce((n, s) => n + s.policies.length, 0) },
          ] as stat}
            <div class="rounded-xl border border-slate-200 bg-white p-4 dark:border-slate-800 dark:bg-panel">
              <div class="text-xs text-slate-500 dark:text-slate-400">{stat.label}</div>
              <div class="mt-1 text-2xl font-semibold tabular-nums tracking-tight">
                {formatNumber(stat.value)}
              </div>
            </div>
          {/each}
          <div class="sm:col-span-2 lg:col-span-4">
            <p class="text-sm text-slate-500 dark:text-slate-400">
              Wähle links eine Tabelle, um ihre Daten zu sehen — oder sichere alles mit
              <span class="font-medium">Backup to Disk</span>. Das Backup umfasst Schema, Daten,
              Views, Funktionen, Trigger, Sequenzen und RLS-Policies.
            </p>
          </div>
        </div>
      {/if}
    </div>
  </div>
{/if}

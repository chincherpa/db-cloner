<script lang="ts">
  import { api, type TablePage } from "../api";
  import { app } from "../stores.svelte";
  import { formatNumber } from "../format";

  const PAGE_SIZE = 100;

  let page = $state(0);
  let sortColumn = $state<string | null>(null);
  let sortDesc = $state(false);
  let data = $state<TablePage | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);

  let current = $derived(app.selectedTable);

  // Reset paging when another table is selected.
  $effect(() => {
    if (current) {
      page = 0;
      sortColumn = null;
      sortDesc = false;
    }
  });

  $effect(() => {
    if (!current || !app.activeId) return;
    const args = {
      id: app.activeId,
      schema: current.schema,
      table: current.table,
      limit: PAGE_SIZE,
      offset: page * PAGE_SIZE,
      sortColumn,
      sortDesc,
    };
    loading = true;
    error = null;
    api
      .getTablePage(args)
      .then((result) => (data = result))
      .catch((e) => (error = String(e)))
      .finally(() => (loading = false));
  });

  function toggleSort(column: string) {
    if (sortColumn === column) {
      if (sortDesc) {
        sortColumn = null;
        sortDesc = false;
      } else {
        sortDesc = true;
      }
    } else {
      sortColumn = column;
      sortDesc = false;
    }
    page = 0;
  }

  const totalPages = $derived(data ? Math.max(1, Math.ceil(data.totalRows / PAGE_SIZE)) : 1);
</script>

{#if current}
  <div class="flex items-center gap-2 border-b border-slate-200 px-4 py-2 dark:border-slate-800">
    <span class="font-mono text-sm">
      <span class="text-slate-400">{current.schema}.</span><span class="font-semibold">{current.table}</span>
    </span>
    {#if data}
      <span class="text-xs text-slate-400">{formatNumber(data.totalRows)} Zeilen</span>
    {/if}
    {#if loading}
      <span class="h-3.5 w-3.5 animate-spin rounded-full border-2 border-accent border-t-transparent"></span>
    {/if}
    <div class="ml-auto flex items-center gap-1 text-sm">
      <button
        class="rounded-md px-2 py-1 hover:bg-slate-100 disabled:opacity-40 dark:hover:bg-slate-800"
        disabled={page === 0}
        onclick={() => page--}
      >
        ←
      </button>
      <span class="tabular-nums text-xs text-slate-500">Seite {page + 1} / {totalPages}</span>
      <button
        class="rounded-md px-2 py-1 hover:bg-slate-100 disabled:opacity-40 dark:hover:bg-slate-800"
        disabled={page + 1 >= totalPages}
        onclick={() => page++}
      >
        →
      </button>
      <button
        class="ml-2 rounded-md px-2 py-1 text-xs text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800"
        onclick={() => (app.selectedTable = null)}
      >
        Schließen ✕
      </button>
    </div>
  </div>

  {#if error}
    <div class="m-4 rounded-lg border border-red-300 bg-red-50 p-3 text-sm text-red-700 dark:border-red-900 dark:bg-red-950/40 dark:text-red-300 selectable">
      {error}
    </div>
  {:else if data}
    <div class="min-h-0 flex-1 overflow-auto selectable">
      <table class="min-w-full border-collapse text-[13px]">
        <thead class="sticky top-0 z-10">
          <tr class="bg-slate-50 dark:bg-panel-2">
            {#each data.columns as col (col.name)}
              <th
                class="cursor-pointer whitespace-nowrap border-b border-slate-200 px-3 py-2 text-left font-medium dark:border-slate-700"
                onclick={() => toggleSort(col.name)}
              >
                <span class="inline-flex items-center gap-1">
                  {col.name}
                  {#if sortColumn === col.name}
                    <span class="text-accent">{sortDesc ? "↓" : "↑"}</span>
                  {/if}
                </span>
                <span class="block text-[10px] font-normal text-slate-400">{col.dataType}</span>
              </th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each data.rows as row, i (i)}
            <tr class="odd:bg-white even:bg-slate-50/60 dark:odd:bg-transparent dark:even:bg-slate-800/30">
              {#each row as cell, j (j)}
                <td
                  class="max-w-96 truncate whitespace-nowrap border-b border-slate-100 px-3 py-1.5 font-mono dark:border-slate-800"
                  title={cell ?? ""}
                >
                  {#if cell === null}
                    <span class="italic text-slate-400">NULL</span>
                  {:else}
                    {cell}
                  {/if}
                </td>
              {/each}
            </tr>
          {/each}
          {#if data.rows.length === 0}
            <tr>
              <td colspan={data.columns.length} class="px-3 py-8 text-center text-sm text-slate-400">
                Diese Tabelle ist leer.
              </td>
            </tr>
          {/if}
        </tbody>
      </table>
    </div>
  {/if}
{/if}

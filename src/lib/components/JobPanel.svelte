<script lang="ts">
  import { api } from "../api";
  import { app, dismissJob } from "../stores.svelte";

  let expanded = $state<Record<string, boolean>>({});

  async function cancel(jobId: string) {
    try {
      await api.cancelJob(jobId);
    } catch {
      // job might already be gone
    }
  }
</script>

{#if app.jobs.length > 0}
  <div class="fixed bottom-4 right-4 z-30 flex w-[430px] max-w-[calc(100vw-2rem)] flex-col gap-3">
    {#each app.jobs as job (job.jobId)}
      <div
        class="overflow-hidden rounded-xl border border-slate-200 bg-white shadow-xl dark:border-slate-700 dark:bg-panel"
      >
        <div class="flex items-center gap-3 px-4 py-3">
          {#if job.status === "running"}
            <span class="h-4 w-4 shrink-0 animate-spin rounded-full border-2 border-accent border-t-transparent"></span>
          {:else if job.status === "done"}
            <span class="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-emerald-500 text-[11px] font-bold text-white">✓</span>
          {:else}
            <span class="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-red-500 text-[11px] font-bold text-white">✕</span>
          {/if}
          <div class="min-w-0 flex-1">
            <div class="truncate text-sm font-medium">
              {job.status === "running"
                ? job.phase || (job.kind === "restore" ? "Restore läuft…" : "Backup läuft…")
                : job.message}
            </div>
            {#if job.status === "running" && job.percent >= 0}
              <div class="mt-1.5 h-1.5 overflow-hidden rounded-full bg-slate-200 dark:bg-slate-700">
                <div
                  class="h-full rounded-full bg-accent transition-all duration-300"
                  style="width: {Math.min(100, job.percent)}%"
                ></div>
              </div>
            {:else if job.status === "running"}
              <div class="mt-1.5 h-1.5 overflow-hidden rounded-full bg-slate-200 dark:bg-slate-700">
                <div class="h-full w-1/3 animate-pulse rounded-full bg-accent"></div>
              </div>
            {/if}
          </div>
          <button
            class="shrink-0 rounded-md px-2 py-1 text-xs text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800"
            onclick={() => (expanded[job.jobId] = !expanded[job.jobId])}
          >
            Log
          </button>
          {#if job.status === "running"}
            <button
              class="shrink-0 rounded-md px-2 py-1 text-xs text-red-500 hover:bg-red-50 dark:hover:bg-red-950/30"
              onclick={() => void cancel(job.jobId)}
            >
              Abbrechen
            </button>
          {:else}
            <button
              class="shrink-0 rounded-md px-2 py-1 text-xs text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800"
              onclick={() => dismissJob(job.jobId)}
            >
              ✕
            </button>
          {/if}
        </div>
        {#if expanded[job.jobId]}
          <div
            class="max-h-48 overflow-y-auto border-t border-slate-200 bg-slate-50 px-4 py-2 font-mono text-[11px] leading-relaxed text-slate-600 dark:border-slate-700 dark:bg-ink dark:text-slate-300 selectable"
          >
            {#each job.lines.slice(-300) as line, i (i)}
              <div class="whitespace-pre-wrap break-all">{line}</div>
            {/each}
            {#if job.lines.length === 0}
              <div class="italic text-slate-400">Noch keine Ausgabe…</div>
            {/if}
          </div>
        {/if}
      </div>
    {/each}
  </div>
{/if}

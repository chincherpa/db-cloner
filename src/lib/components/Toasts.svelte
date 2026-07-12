<script lang="ts">
  import { app } from "../stores.svelte";
</script>

{#if app.toasts.length > 0}
  <div class="fixed right-4 top-4 z-50 flex w-96 max-w-[calc(100vw-2rem)] flex-col gap-2">
    {#each app.toasts as toast (toast.id)}
      <div
        class="rounded-xl border px-4 py-3 text-sm shadow-lg backdrop-blur
               {toast.kind === 'success'
          ? 'border-emerald-300 bg-emerald-50/95 text-emerald-800 dark:border-emerald-800 dark:bg-emerald-950/90 dark:text-emerald-200'
          : toast.kind === 'error'
            ? 'border-red-300 bg-red-50/95 text-red-800 dark:border-red-900 dark:bg-red-950/90 dark:text-red-200'
            : 'border-slate-200 bg-white/95 text-slate-700 dark:border-slate-700 dark:bg-panel/95 dark:text-slate-200'}"
      >
        {toast.message}
      </div>
    {/each}
  </div>
{/if}

<script lang="ts">
  import { scanStore } from '../stores/scan';

  $: stats = $scanStore.planStats;
  $: plan = $scanStore.plan;
</script>

<div class="container mx-auto p-8">
  <div class="card space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <p class="text-sm uppercase tracking-wide text-primary-600 font-semibold">Phase 4.4 preview</p>
        <h1 class="text-3xl font-bold">Cleanup Plan</h1>
        <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
          Shows the generated plan from the latest scan; editing to come.
        </p>
      </div>
    </div>

    <div class="flex gap-3">
      <button class="btn-primary" disabled={!plan}>
        Save Plan
      </button>
      <button class="btn-secondary" disabled={!plan}>
        Export
      </button>
    </div>

    {#if stats && plan}
      <div>
        <h2 class="text-xl font-semibold mb-3">Plan Summary</h2>
        <div class="grid grid-cols-3 gap-4">
          <div class="p-3 bg-red-50 dark:bg-red-900/20 rounded-lg">
            <div class="text-xs text-gray-600 dark:text-gray-400">Delete</div>
            <div class="text-xl font-bold text-red-600 dark:text-red-400">{stats.delete_count}</div>
          </div>

          <div class="p-3 bg-yellow-50 dark:bg-yellow-900/20 rounded-lg">
            <div class="text-xs text-gray-600 dark:text-gray-400">Review</div>
            <div class="text-xl font-bold text-yellow-600 dark:text-yellow-400">{stats.review_count}</div>
          </div>

          <div class="p-3 bg-green-50 dark:bg-green-900/20 rounded-lg">
            <div class="text-xs text-gray-600 dark:text-gray-400">Keep</div>
            <div class="text-xl font-bold text-green-600 dark:text-green-400">{stats.keep_count}</div>
          </div>
        </div>
      </div>

      <div>
        <h2 class="text-xl font-semibold mb-3">Entries</h2>
        <div class="p-4 bg-gray-50 dark:bg-gray-700 rounded-lg text-sm text-gray-700 dark:text-gray-200 space-y-2 max-h-64 overflow-auto">
          {#each plan.entries.slice(0, 10) as entry}
            <div class="border-b border-gray-200 dark:border-gray-700 pb-2 mb-2 last:border-0 last:pb-0 last:mb-0">
              <div class="font-medium">{entry.path}</div>
              <div class="text-xs text-gray-500 dark:text-gray-400 flex gap-2">
                <span>{entry.action}</span>
                <span>{entry.rule_name}</span>
              </div>
              <div class="text-xs text-gray-500 dark:text-gray-400">{entry.reason}</div>
            </div>
          {/each}
          {#if plan.entries.length > 10}
            <div class="text-xs text-gray-500 dark:text-gray-400">
              …and {plan.entries.length - 10} more (scroll or export to view all)
            </div>
          {/if}
        </div>
      </div>

      <div class="p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg">
        <p class="text-sm text-yellow-800 dark:text-yellow-200">
          Editing (batch actions, save/load) will be added in Phase 4.4.
        </p>
      </div>
    {:else}
      <div class="p-8 bg-gray-50 dark:bg-gray-700 rounded-lg text-center">
        <p class="text-gray-500 dark:text-gray-400">
          No cleanup plan loaded. Generate a plan from the Scan page to populate this view.
        </p>
      </div>
    {/if}
  </div>
</div>

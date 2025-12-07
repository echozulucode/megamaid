<script lang="ts">
  import { scanStore } from '../stores/scan';
  import {
    getPlanStats,
    loadPlanFromFile,
    savePlanToFile,
    type CleanupAction,
    type CleanupEntry,
  } from '../services/tauri';
  import { isProtectedPath } from '../utils/pathProtection';

  $: stats = $scanStore.planStats;
  $: plan = $scanStore.plan;
  let actionFilter: 'all' | 'delete' | 'review' | 'keep' = 'all';
  let planError: string | null = null;
  let saving = false;
  let loading = false;
  let infoMessage: string | null = null;
  let showActions = true;
  let batchMessage: string | null = null;

  $: filteredEntries = plan
    ? plan.entries.filter((entry) => {
        if (actionFilter === 'all') return true;
        return entry.action === actionFilter;
      })
    : [];

  function updateAction(path: string, action: CleanupAction) {
    if (!plan) return;
    if (action === 'delete' && isProtectedPath(path)) {
        planError = 'Delete is disabled for protected paths (repo roots/manifests).';
        return;
    }
    const updatedEntries = plan.entries.map((entry) =>
      entry.path === path ? { ...entry, action } : entry
    );
    const updatedPlan = { ...plan, entries: updatedEntries };
    const updatedStats = computeStats(updatedPlan.entries);
    scanStore.update((s) => ({
      ...s,
      plan: updatedPlan,
      planStats: updatedStats,
    }));
  }

  function computeStats(entries: CleanupEntry[]) {
    const delete_count = entries.filter((e) => e.action === 'delete').length;
    const review_count = entries.filter((e) => e.action === 'review').length;
    const keep_count = entries.filter((e) => e.action === 'keep').length;
    const total_size = entries.reduce((acc, e) => acc + e.size, 0);
    return {
      total_entries: entries.length,
      delete_count,
      review_count,
      keep_count,
      total_size,
    };
  }

  async function handleLoadPlan() {
    planError = null;
    infoMessage = null;
    loading = true;
    try {
      const loaded = await loadPlanFromFile();
      if (loaded) {
        const loadedStats = await getPlanStats(loaded);
        scanStore.update((s) => ({
          ...s,
          plan: loaded,
          planStats: loadedStats,
          directory: loaded.base_path?.toString() ?? s.directory,
        }));
        infoMessage = 'Plan loaded successfully.';
      }
    } catch (err) {
      planError = err instanceof Error ? err.message : String(err);
    } finally {
      loading = false;
    }
  }

  async function handleSavePlan() {
    planError = null;
    infoMessage = null;
    if (!plan) return;
    saving = true;
    try {
      await savePlanToFile(plan);
      infoMessage = 'Plan saved to disk.';
    } catch (err) {
      planError = err instanceof Error ? err.message : String(err);
    } finally {
      saving = false;
    }
  }

  function applyBatch(rule: 'build_artifact' | 'source_keep') {
    if (!plan) return;
    planError = null;
    batchMessage = null;
    const updatedEntries = plan.entries.map((entry) => {
      if (rule === 'build_artifact' && entry.rule_name === 'build_artifact' && !isProtectedPath(entry.path)) {
        return { ...entry, action: 'delete' as CleanupAction };
      }
      if (rule === 'source_keep' && isProtectedPath(entry.path)) {
        return { ...entry, action: 'keep' as CleanupAction };
      }
      return entry;
    });
    const updatedPlan = { ...plan, entries: updatedEntries };
    const updatedStats = computeStats(updatedPlan.entries);
    scanStore.update((s) => ({
      ...s,
      plan: updatedPlan,
      planStats: updatedStats,
    }));
    batchMessage = 'Batch actions applied.';
  }
</script>

<div class="container mx-auto p-8">
    <div class="card space-y-6">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-3xl font-bold">Cleanup Plan</h1>
          <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
            Review and adjust actions, then save or load plans from disk.
          </p>
        </div>
        {#if plan}
          <button class="btn-secondary" on:click={() => (showActions = !showActions)}>
            {showActions ? 'Hide Actions' : 'Show Actions'}
          </button>
        {/if}
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
        <div class="flex items-center gap-3 mb-3 text-sm">
          <label class="text-gray-600 dark:text-gray-400" for="plan-filter">Filter:</label>
          <select
            id="plan-filter"
            class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700"
            bind:value={actionFilter}
          >
            <option value="all">All</option>
            <option value="delete">Delete</option>
            <option value="review">Review</option>
            <option value="keep">Keep</option>
          </select>
        </div>

        <div class="p-4 bg-gray-50 dark:bg-gray-700 rounded-lg text-sm text-gray-700 dark:text-gray-200 space-y-2 max-h-64 overflow-auto">
          {#each filteredEntries.slice(0, 10) as entry}
            <div class="border-b border-gray-200 dark:border-gray-700 pb-2 mb-2 last:border-0 last:pb-0 last:mb-0">
              <div class="font-medium">{entry.path}</div>
              <div class="text-xs text-gray-500 dark:text-gray-400 flex gap-2">
                <span>{entry.action}</span>
                <span>{entry.rule_name}</span>
              </div>
              <div class="text-xs text-gray-500 dark:text-gray-400">{entry.reason}</div>
              {#if showActions}
                <div class="flex gap-2 mt-2">
                  <button class="btn-secondary text-xs" on:click={() => updateAction(entry.path, 'keep')}>
                    Keep
                  </button>
                  <button class="btn-secondary text-xs" on:click={() => updateAction(entry.path, 'review')}>
                    Review
                  </button>
                  <button
                    class="btn-primary text-xs"
                    on:click={() => updateAction(entry.path, 'delete')}
                  >
                    Delete
                  </button>
                </div>
              {/if}
            </div>
          {/each}
          {#if filteredEntries.length > 10}
            <div class="text-xs text-gray-500 dark:text-gray-400">
              …and {filteredEntries.length - 10} more (scroll or export to view all)
            </div>
          {/if}
        </div>
      </div>
    {:else}
      <div class="p-8 bg-gray-50 dark:bg-gray-700 rounded-lg text-center">
        <p class="text-gray-500 dark:text-gray-400">
          No cleanup plan loaded. Generate a plan from the Scan page to populate this view.
        </p>
        <button class="btn-secondary mt-3" on:click={handleLoadPlan} disabled={loading}>
          {loading ? 'Loading…' : 'Load Plan'}
        </button>
      </div>
    {/if}

    {#if plan}
      <div class="flex gap-3">
        <button class="btn-primary" on:click={handleSavePlan} disabled={saving}>
          {saving ? 'Saving…' : 'Save Plan'}
        </button>
        <button class="btn-secondary" on:click={handleLoadPlan} disabled={loading}>
          {loading ? 'Loading…' : 'Load Plan'}
        </button>
      </div>
    {/if}

    {#if planError}
      <div class="p-3 border border-red-200 dark:border-red-700 bg-red-50 dark:bg-red-900/30 rounded-lg text-sm text-red-800 dark:text-red-100">
        {planError}
      </div>
    {/if}

    {#if infoMessage}
      <div class="p-3 border border-green-200 dark:border-green-700 bg-green-50 dark:bg-green-900/30 rounded-lg text-sm text-green-800 dark:text-green-100">
        {infoMessage}
      </div>
    {/if}
  </div>
</div>

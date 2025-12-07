<script lang="ts">
  import { scanStore } from '../stores/scan';

  function formatBytes(bytes: number): string {
    if (!bytes) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    const value = bytes / Math.pow(1024, i);
    return `${value.toFixed(1)} ${units[i]}`;
  }

  $: stats = $scanStore.planStats;
  $: scan = $scanStore.scanResult;
</script>

<div class="container mx-auto p-8">
  <div class="card space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <p class="text-sm uppercase tracking-wide text-primary-600 font-semibold">Phase 4.2 preview</p>
        <h1 class="text-3xl font-bold">Scan Results</h1>
        <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
          Basic readout from the latest scan and plan.
        </p>
      </div>
    </div>

    <div class="grid grid-cols-3 gap-4 mb-2">
      <div class="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
        <div class="text-sm text-gray-600 dark:text-gray-400">Files Scanned</div>
        <div class="text-2xl font-bold">{(scan?.total_files ?? 0).toLocaleString()}</div>
      </div>

      <div class="p-4 bg-yellow-50 dark:bg-yellow-900/20 rounded-lg">
        <div class="text-sm text-gray-600 dark:text-gray-400">Cleanup Candidates</div>
        <div class="text-2xl font-bold">{stats ? stats.total_entries.toLocaleString() : 0}</div>
      </div>

      <div class="p-4 bg-green-50 dark:bg-green-900/20 rounded-lg">
        <div class="text-sm text-gray-600 dark:text-gray-400">Potential Space Saved</div>
        <div class="text-2xl font-bold">{stats ? formatBytes(stats.total_size) : '0 B'}</div>
      </div>
    </div>

    {#if stats && scan}
      <div class="space-y-4">
        <h2 class="text-xl font-semibold">Summary</h2>

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

        <div class="p-4 bg-gray-50 dark:bg-gray-700 rounded-lg text-sm text-gray-700 dark:text-gray-200">
          <p class="font-medium">Base Path</p>
          <p class="mt-1">{stats ? $scanStore.plan?.base_path ?? 'Unknown' : 'No plan loaded'}</p>
        </div>

        <div class="p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg text-sm text-yellow-800 dark:text-yellow-200">
          Visual tree/treemap will be added in Phase 4.3.
        </div>
      </div>
    {:else}
      <div class="p-8 bg-gray-50 dark:bg-gray-700 rounded-lg text-center">
        <p class="text-gray-500 dark:text-gray-400">
          No scan results available. Run a scan from the Scan page to populate this view.
        </p>
      </div>
    {/if}
  </div>
</div>

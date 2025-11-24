<script lang="ts">
  // Scan interface will be implemented in Phase 4.2
</script>

<div class="container mx-auto p-8">
  <div class="card">
    <h1 class="text-3xl font-bold mb-6">Scan Directory</h1>

    <div class="space-y-6">
      <div>
        <label class="block text-sm font-medium mb-2">
          Select Directory
        </label>
        <div class="flex gap-2">
          <input
            type="text"
            placeholder="Choose a directory to scan..."
            class="flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700"
            readonly
          />
          <button class="btn-primary">
            Browse
          </button>
        </div>
      </div>

      <div>
        <label class="block text-sm font-medium mb-2">
          Scan Options
        </label>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block text-xs text-gray-600 dark:text-gray-400 mb-1">
              Max Depth
            </label>
            <input
              type="number"
              value="10"
              min="1"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700"
            />
          </div>
          <div>
            <label class="block text-xs text-gray-600 dark:text-gray-400 mb-1">
              Large File Threshold (MB)
            </label>
            <input
              type="number"
              value="100"
              min="1"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700"
            />
          </div>
        </div>

        <div class="mt-3">
          <label class="flex items-center gap-2">
            <input type="checkbox" checked class="rounded" />
            <span class="text-sm">Skip hidden files and directories</span>
          </label>
        </div>
      </div>

      <div class="flex gap-3">
        <button class="btn-primary" disabled>
          Start Scan
        </button>
        <button class="btn-secondary">
          Cancel
        </button>
      </div>

      <div class="p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg">
        <p class="text-sm text-yellow-800 dark:text-yellow-200">
          <strong>Note:</strong> Scan functionality will be implemented in Phase 4.2
        </p>
      </div>
    </div>
  </div>
</div>

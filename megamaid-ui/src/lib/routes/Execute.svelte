<script lang="ts">
  // Execution monitoring will be implemented in Phase 4.5
</script>

<div class="container mx-auto p-8">
  <div class="card">
    <h1 class="text-3xl font-bold mb-6">Execute Cleanup</h1>

    <div class="space-y-6">
      <div>
        <h2 class="text-xl font-semibold mb-3">Execution Mode</h2>
        <div class="space-y-2">
          <label class="flex items-center gap-2 p-3 border border-gray-300 dark:border-gray-600 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700">
            <input type="radio" name="mode" value="dry-run" checked />
            <div>
              <div class="font-medium">Dry Run</div>
              <div class="text-sm text-gray-600 dark:text-gray-400">Preview without actual deletion</div>
            </div>
          </label>

          <label class="flex items-center gap-2 p-3 border border-gray-300 dark:border-gray-600 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700">
            <input type="radio" name="mode" value="backup" />
            <div>
              <div class="font-medium">Backup Mode</div>
              <div class="text-sm text-gray-600 dark:text-gray-400">Move files to backup directory</div>
            </div>
          </label>

          <label class="flex items-center gap-2 p-3 border border-gray-300 dark:border-gray-600 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700">
            <input type="radio" name="mode" value="recycle" />
            <div>
              <div class="font-medium">Recycle Bin</div>
              <div class="text-sm text-gray-600 dark:text-gray-400">Move files to recycle bin</div>
            </div>
          </label>

          <label class="flex items-center gap-2 p-3 border border-gray-300 dark:border-gray-600 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700">
            <input type="radio" name="mode" value="delete" />
            <div>
              <div class="font-medium">Direct Delete</div>
              <div class="text-sm text-gray-600 dark:text-gray-400 text-red-600 dark:text-red-400">
                Permanent deletion - cannot be undone
              </div>
            </div>
          </label>
        </div>
      </div>

      <div>
        <h2 class="text-xl font-semibold mb-3">Options</h2>
        <div class="space-y-2">
          <label class="flex items-center gap-2">
            <input type="checkbox" checked class="rounded" />
            <span class="text-sm">Verify plan before execution</span>
          </label>
          <label class="flex items-center gap-2">
            <input type="checkbox" class="rounded" />
            <span class="text-sm">Fail-fast mode (stop on first error)</span>
          </label>
        </div>
      </div>

      <div class="flex gap-3">
        <button class="btn-primary" disabled>
          Start Execution
        </button>
        <button class="btn-secondary">
          Cancel
        </button>
      </div>

      <div class="p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg">
        <p class="text-sm text-yellow-800 dark:text-yellow-200">
          <strong>Note:</strong> Execution monitoring will be implemented in Phase 4.5
        </p>
      </div>
    </div>
  </div>
</div>

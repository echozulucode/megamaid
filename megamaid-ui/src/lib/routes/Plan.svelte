<script lang="ts">
  // Plan editor will be implemented in Phase 4.4
</script>

<div class="container mx-auto p-8">
  <div class="card">
    <h1 class="text-3xl font-bold mb-6">Cleanup Plan</h1>

    <div class="space-y-6">
      <div class="flex gap-3">
        <button class="btn-primary">
          Load Plan
        </button>
        <button class="btn-secondary">
          Save Plan
        </button>
        <button class="btn-secondary">
          Export
        </button>
      </div>

      <div>
        <h2 class="text-xl font-semibold mb-3">Plan Summary</h2>
        <div class="grid grid-cols-3 gap-4">
          <div class="p-3 bg-red-50 dark:bg-red-900/20 rounded-lg">
            <div class="text-xs text-gray-600 dark:text-gray-400">Delete</div>
            <div class="text-xl font-bold text-red-600 dark:text-red-400">0</div>
          </div>

          <div class="p-3 bg-yellow-50 dark:bg-yellow-900/20 rounded-lg">
            <div class="text-xs text-gray-600 dark:text-gray-400">Review</div>
            <div class="text-xl font-bold text-yellow-600 dark:text-yellow-400">0</div>
          </div>

          <div class="p-3 bg-green-50 dark:bg-green-900/20 rounded-lg">
            <div class="text-xs text-gray-600 dark:text-gray-400">Keep</div>
            <div class="text-xl font-bold text-green-600 dark:text-green-400">0</div>
          </div>
        </div>
      </div>

      <div>
        <h2 class="text-xl font-semibold mb-3">Entries</h2>
        <div class="p-8 bg-gray-50 dark:bg-gray-700 rounded-lg text-center">
          <p class="text-gray-500 dark:text-gray-400">
            No cleanup plan loaded. Generate a plan from scan results or load an existing plan.
          </p>
        </div>
      </div>

      <div class="p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg">
        <p class="text-sm text-yellow-800 dark:text-yellow-200">
          <strong>Note:</strong> Plan editing will be implemented in Phase 4.4
        </p>
      </div>
    </div>
  </div>
</div>

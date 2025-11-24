<script lang="ts">
  import { link } from 'svelte-spa-router';
</script>

<div class="flex flex-col items-center justify-center min-h-screen p-8">
  <div class="card max-w-2xl w-full text-center">
    <h1 class="text-4xl font-bold mb-4 text-primary-600 dark:text-primary-400">
      Megamaid
    </h1>
    <p class="text-xl text-gray-600 dark:text-gray-300 mb-8">
      High-Performance Storage Analysis & Cleanup Tool
    </p>

    <div class="grid gap-4 mb-8">
      <div class="p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
        <h2 class="text-lg font-semibold mb-2">Scan Directories</h2>
        <p class="text-sm text-gray-600 dark:text-gray-400">
          Analyze your file system to identify cleanup candidates
        </p>
      </div>

      <div class="p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
        <h2 class="text-lg font-semibold mb-2">Review Plans</h2>
        <p class="text-sm text-gray-600 dark:text-gray-400">
          Edit and verify cleanup plans before execution
        </p>
      </div>

      <div class="p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
        <h2 class="text-lg font-semibold mb-2">Execute Safely</h2>
        <p class="text-sm text-gray-600 dark:text-gray-400">
          Delete files with drift detection and transaction logging
        </p>
      </div>
    </div>

    <a href="/scan" use:link class="btn-primary inline-block">
      Start Scan
    </a>
  </div>
</div>

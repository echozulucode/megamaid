<script lang="ts">
  import { location } from 'svelte-spa-router';

  const navItems = [
    { path: '/', label: 'Home', icon: '🏠' },
    { path: '/scan', label: 'Scan', icon: '🔍' },
    { path: '/results', label: 'Results', icon: '📊' },
    { path: '/plan', label: 'Plan', icon: '📝' },
    { path: '/execute', label: 'Execute', icon: '⚡' },
  ];

  function isActive(path: string): boolean {
    return $location === path;
  }
</script>

<div class="flex h-screen bg-gray-50 dark:bg-gray-900">
  <!-- Sidebar Navigation -->
  <nav class="w-64 bg-white dark:bg-gray-800 shadow-lg">
    <div class="p-6">
      <h1 class="text-2xl font-bold text-primary-600 dark:text-primary-400">
        Megamaid
      </h1>
      <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
        Storage Cleanup Tool
      </p>
    </div>

    <ul class="space-y-1 px-3">
      {#each navItems as item}
        <li>
          <a
            href="#{item.path}"
            class="flex items-center gap-3 px-4 py-3 rounded-lg transition-colors {isActive(item.path)
              ? 'bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300 font-medium'
              : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'}"
          >
            <span class="text-xl">{item.icon}</span>
            <span>{item.label}</span>
          </a>
        </li>
      {/each}
    </ul>

    <div class="absolute bottom-0 left-0 right-0 p-4 border-t border-gray-200 dark:border-gray-700 w-64">
      <div class="text-xs text-gray-500 dark:text-gray-400">
        <div>Phase 4.1 - Setup</div>
        <div class="mt-1 text-primary-600 dark:text-primary-400">
          In Development
        </div>
      </div>
    </div>
  </nav>

  <!-- Main Content Area -->
  <main class="flex-1 overflow-auto">
    <slot />
  </main>
</div>

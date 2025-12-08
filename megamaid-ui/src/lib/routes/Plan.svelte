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

  type TreeNode = {
    name: string;
    path: string;
    depth: number;
    children: TreeNode[];
    entries: CleanupEntry[];
    size: number;
    deleteCount: number;
    reviewCount: number;
    keepCount: number;
  };

  type TreeRow =
    | {
        kind: 'dir';
        path: string;
        name: string;
        depth: number;
        size: number;
        deleteCount: number;
        reviewCount: number;
        keepCount: number;
      }
    | {
        kind: 'entry';
        path: string;
        name: string;
        depth: number;
        entry: CleanupEntry;
      };

  $: plan = $scanStore.plan;
  $: stats = $scanStore.planStats;
  let planError: string | null = null;
  let infoMessage: string | null = null;
  let saving = false;
  let loading = false;
  let actionFilter: 'all' | 'delete' | 'review' | 'keep' = 'all';
  let search = '';
  let pendingDeletes = false;
  let activeTab: 'tree' | 'list' = 'tree';
  let expandedNodes: Set<string> = new Set(['__root__']);
  let selectedPath: string | null = null;
  let entries: CleanupEntry[] = [];
  let visibleEntries: CleanupEntry[] = [];
  let treeRoot: TreeNode | null = null;
  let treeMap: Map<string, TreeNode> = new Map();
  let treeRows: TreeRow[] = [];
  let currentEntry: CleanupEntry | undefined;
  let currentDir: TreeNode | null = null;

  const formatBytes = (bytes: number): string => {
    if (!bytes) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    const value = bytes / Math.pow(1024, i);
    return `${value.toFixed(1)} ${units[i]}`;
  };

  const normalizePath = (path: string): string => path.replace(/\\/g, '/');

  function buildTree(entries: CleanupEntry[]): { root: TreeNode; map: Map<string, TreeNode> } {
    const root: TreeNode = {
      name: 'Root',
      path: '__root__',
      depth: 0,
      children: [],
      entries: [],
      size: 0,
      deleteCount: 0,
      reviewCount: 0,
      keepCount: 0,
    };

    const nodeMap = new Map<string, TreeNode>();
    nodeMap.set(root.path, root);

    for (const entry of entries) {
      const normalized = normalizePath(entry.path);
      if (normalized === '.') continue;
      const parts = normalized.split('/').filter(Boolean);
      let current = root;
      let accumulated = '';

      for (const part of parts.slice(0, -1)) {
        accumulated = accumulated ? `${accumulated}/${part}` : part;
        let child = nodeMap.get(accumulated);
        if (!child) {
          child = {
            name: part,
            path: accumulated,
            depth: current.depth + 1,
            children: [],
            entries: [],
            size: 0,
            deleteCount: 0,
            reviewCount: 0,
            keepCount: 0,
          };
          current.children.push(child);
          nodeMap.set(accumulated, child);
        }
        current = child;
      }

      current.entries.push(entry);
    }

    const compute = (node: TreeNode): TreeNode => {
      let size = 0;
      let deleteCount = 0;
      let reviewCount = 0;
      let keepCount = 0;

      for (const child of node.children) {
        const c = compute(child);
        size += c.size;
        deleteCount += c.deleteCount;
        reviewCount += c.reviewCount;
        keepCount += c.keepCount;
      }

      for (const e of node.entries) {
        size += e.size;
        if (e.action === 'delete') deleteCount += 1;
        if (e.action === 'review') reviewCount += 1;
        if (e.action === 'keep') keepCount += 1;
      }

      node.size = size;
      node.deleteCount = deleteCount;
      node.reviewCount = reviewCount;
      node.keepCount = keepCount;
      return node;
    };

    compute(root);
    return { root, map: nodeMap };
  }

  function flattenTree(node: TreeNode, expanded: Set<string>): TreeRow[] {
    const rows: TreeRow[] = [];
    const isRoot = node.path === '__root__';
    if (!isRoot) {
      rows.push({
        kind: 'dir',
        path: node.path,
        name: node.name,
        depth: node.depth,
        size: node.size,
        deleteCount: node.deleteCount,
        reviewCount: node.reviewCount,
        keepCount: node.keepCount,
      });
    }

    if (isRoot || expanded.has(node.path)) {
      for (const child of node.children) {
        rows.push(...flattenTree(child, expanded));
      }
      for (const entry of node.entries) {
        const depth = isRoot ? 1 : node.depth + 1;
        const parts = normalizePath(entry.path).split('/').filter(Boolean);
        const name = parts[parts.length - 1] ?? entry.path;
        rows.push({
          kind: 'entry',
          path: entry.path,
          name,
          depth,
          entry,
        });
      }
    }

    return rows;
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

  $: entries = plan ? plan.entries.filter((entry) => entry.path !== '.') : [];
  $: visibleEntries = entries.filter((entry) => {
    if (pendingDeletes && entry.action !== 'delete') return false;
    if (actionFilter !== 'all' && entry.action !== actionFilter) return false;
    if (search && !entry.path.toLowerCase().includes(search.toLowerCase())) return false;
    return true;
  });
  $: ({ root: treeRoot, map: treeMap } = buildTree(visibleEntries));
  $: treeRows = flattenTree(treeRoot, expandedNodes);
  // Resolve selection against the full plan (not just filtered view) so details still show.
  $: currentEntry = plan?.entries.find((e) => e.path === selectedPath);
  $: currentDir = selectedPath ? treeMap.get(selectedPath) ?? null : null;

  function focusPath(path: string) {
    const normalized = normalizePath(path);
    const parts = normalized.split('/').filter(Boolean);
    const next = new Set(expandedNodes);
    let acc = '';
    for (const part of parts.slice(0, -1)) {
      acc = acc ? `${acc}/${part}` : part;
      next.add(acc);
    }
    expandedNodes = next;
    selectedPath = path;
    activeTab = 'tree';
  }

  function updatePlanEntries(updatedEntries: CleanupEntry[]) {
    if (!plan) return;
    const updatedPlan = { ...plan, entries: updatedEntries };
    const updatedStats = computeStats(updatedEntries);
    scanStore.update((s) => ({
      ...s,
      plan: updatedPlan,
      planStats: updatedStats,
    }));
  }

  function updateAction(path: string, action: CleanupAction) {
    if (!plan) return;
    if (action === 'delete' && isProtectedPath(path)) {
      planError = 'Delete is disabled for protected paths (repo roots/manifests).';
      return;
    }
    const updatedEntries = plan.entries.map((entry) =>
      entry.path === path ? { ...entry, action } : entry
    );
    updatePlanEntries(updatedEntries);
    infoMessage = `Updated ${path} -> ${action}.`;
  }

  function updateSubtreeAction(path: string, action: CleanupAction) {
    if (!plan) return;
    planError = null;
    const normalized = normalizePath(path);
    const updatedEntries = plan.entries.map((entry) => {
      const entryPath = normalizePath(entry.path);
      const isTarget =
        entryPath === normalized || entryPath.startsWith(`${normalized}/`);
      if (!isTarget) return entry;
      if (action === 'delete' && isProtectedPath(entry.path)) {
        return entry;
      }
      return { ...entry, action };
    });
    updatePlanEntries(updatedEntries);
    infoMessage = `Applied ${action} to ${path} subtree.`;
  }

  function toggleDirExpand(path: string) {
    const next = new Set(expandedNodes);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    expandedNodes = next;
  }

  const handleDirKey = (event: KeyboardEvent, path: string) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      toggleDirExpand(path);
      selectedPath = path;
    }
  };

  async function handleLoadPlan() {
    planError = null;
    infoMessage = null;
    loading = true;
    try {
      const loaded = await loadPlanFromFile();
      if (loaded) {
        const cleanedEntries = loaded.entries.filter((entry) => entry.path !== '.');
        const normalizedPlan = { ...loaded, entries: cleanedEntries };
        const loadedStats = await getPlanStats(normalizedPlan);
        scanStore.update((s) => ({
          ...s,
          plan: normalizedPlan,
          planStats: loadedStats,
          directory: loaded.base_path?.toString() ?? s.directory,
        }));
        infoMessage = 'Plan loaded successfully.';
        selectedPath = null;
        expandedNodes = new Set(['__root__']);
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
      const savedPath = await savePlanToFile(plan);
      infoMessage = savedPath ? `Plan saved to ${savedPath}.` : 'Save canceled.';
    } catch (err) {
      planError = err instanceof Error ? err.message : String(err);
    } finally {
      saving = false;
    }
  }

  function applyBatch(rule: 'build_artifact' | 'source_keep') {
    if (!plan) return;
    planError = null;
    const updatedEntries = plan.entries.map((entry) => {
      if (entry.path === '.') return entry;
      if (rule === 'build_artifact' && entry.rule_name === 'build_artifact' && !isProtectedPath(entry.path)) {
        return { ...entry, action: 'delete' as CleanupAction };
      }
      if (rule === 'source_keep' && isProtectedPath(entry.path)) {
        return { ...entry, action: 'keep' as CleanupAction };
      }
      return entry;
    });
    updatePlanEntries(updatedEntries);
    infoMessage = 'Batch actions applied.';
  }

  const statusBanner = (action: CleanupAction | undefined) => {
    if (!action) return '';
    if (action === 'delete') return 'This item will be deleted.';
    if (action === 'keep') return 'This item will be kept.';
    return 'This item needs review.';
  };
</script>

<div class="container mx-auto p-8">
  <div class="card space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <p class="text-sm uppercase tracking-wide text-primary-600 font-semibold">Manage Plan</p>
        <h1 class="text-3xl font-bold">Plan Review & Actions</h1>
        <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
          Tree-first view to see exactly what will be deleted, kept, or reviewed.
        </p>
      </div>
      {#if plan}
        <div class="flex gap-2">
          <button class="btn-primary" on:click={handleSavePlan} disabled={saving}>
            {saving ? 'Saving...' : 'Save Plan'}
          </button>
          <button class="btn-secondary" on:click={handleLoadPlan} disabled={loading}>
            {loading ? 'Loading...' : 'Load Plan'}
          </button>
        </div>
      {/if}
    </div>

    {#if stats && plan}
      <div>
        <h2 class="text-xl font-semibold mb-3">Plan Summary</h2>
        <div class="grid grid-cols-4 gap-4">
          <div class="p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
            <div class="text-xs text-gray-600 dark:text-gray-400">Base Path</div>
            <div class="text-xs text-gray-800 dark:text-gray-100 truncate max-w-xs">
              {plan.base_path ?? 'Unknown'}
            </div>
          </div>
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

      <div class="flex flex-wrap gap-3 text-sm items-center">
        <label class="text-gray-600 dark:text-gray-400" for="action-filter">Filter:</label>
        <select
          id="action-filter"
          class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700"
          bind:value={actionFilter}
        >
          <option value="all">All</option>
          <option value="delete">Delete</option>
          <option value="review">Review</option>
          <option value="keep">Keep</option>
        </select>
        <input
          class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 flex-1 min-w-[200px]"
          placeholder="Search path..."
          bind:value={search}
          type="text"
        />
        <label class="flex items-center gap-2 text-gray-600 dark:text-gray-300">
          <input type="checkbox" bind:checked={pendingDeletes} />
          Pending deletes only
        </label>
        <button class="btn-secondary text-xs" on:click={() => applyBatch('build_artifact')} disabled={!plan}>
          Delete all build artifacts
        </button>
        <button class="btn-secondary text-xs" on:click={() => applyBatch('source_keep')} disabled={!plan}>
          Keep all protected paths
        </button>
      </div>

      <div class="flex gap-3 text-sm mt-2">
        <button class={activeTab === 'tree' ? 'btn-primary' : 'btn-secondary'} on:click={() => (activeTab = 'tree')}>
          Tree
        </button>
        <button class={activeTab === 'list' ? 'btn-primary' : 'btn-secondary'} on:click={() => (activeTab = 'list')}>
          List
        </button>
      </div>

      {#if activeTab === 'tree'}
        <div class="grid grid-cols-3 gap-4 mt-4">
          <div class="col-span-2">
            <div class="p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 max-h-[520px] overflow-auto text-sm space-y-2">
              {#if treeRows.length === 0}
                <div class="text-gray-500 dark:text-gray-400">No entries to display. Run a scan and generate a plan.</div>
              {:else}
                {#each treeRows as row}
                  {#if row.kind === 'dir'}
                    <div
                      role="button"
                      tabindex="0"
                      class={`flex items-center justify-between py-1 px-2 rounded hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer ${selectedPath === row.path ? 'bg-primary-50 dark:bg-primary-900/20' : ''}`}
                      style={`margin-left: ${row.depth * 12}px`}
                      on:click={() => { toggleDirExpand(row.path); selectedPath = row.path; }}
                      on:keydown={(event) => handleDirKey(event, row.path)}
                    >
                      <div class="flex items-center gap-2">
                        <button
                          class="text-xs px-2 py-1 rounded bg-gray-100 dark:bg-gray-700"
                          on:click|stopPropagation={() => {
                            toggleDirExpand(row.path);
                            selectedPath = row.path;
                          }}
                        >
                          {expandedNodes.has(row.path) ? '-' : '+'}
                        </button>
                        <span class="font-medium">{row.name}</span>
                        <span class="text-[10px] px-2 py-1 rounded bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300">
                          {formatBytes(row.size)}
                        </span>
                      </div>
                      <div class="flex gap-2 text-[10px]">
                        <span class="px-2 py-1 rounded bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-200">
                          {row.deleteCount} delete
                        </span>
                        <span class="px-2 py-1 rounded bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-200">
                          {row.reviewCount} review
                        </span>
                        <span class="px-2 py-1 rounded bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-200">
                          {row.keepCount} keep
                        </span>
                      </div>
                    </div>
                  {:else}
                    <div
                      role="button"
                      tabindex="0"
                      class={`flex items-center justify-between py-1 px-2 rounded border border-gray-100 dark:border-gray-700 bg-gray-50 dark:bg-gray-900/40 cursor-pointer ${selectedPath === row.path ? 'ring-1 ring-primary-400' : ''}`}
                      style={`margin-left: ${row.depth * 12}px`}
                      on:click={() => { selectedPath = row.path; activeTab = 'tree'; }}
                      on:keydown={(event) => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); selectedPath = row.path; activeTab = 'tree'; } }}
                    >
                      <div class="flex items-center gap-2">
                        <span class="font-medium">{row.name}</span>
                        {#if isProtectedPath(row.entry.path)}
                          <span class="px-2 py-1 text-[10px] rounded bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-200">
                            Protected
                          </span>
                        {/if}
                        <span class="text-[10px] px-2 py-1 rounded bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-300">
                          {formatBytes(row.entry.size)}
                        </span>
                      </div>
                      <div class="flex items-center gap-2 text-[10px]">
                        <span class={`px-2 py-1 rounded uppercase tracking-wide ${row.entry.action === 'delete'
                          ? 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-200'
                          : row.entry.action === 'review'
                          ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-200'
                          : 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-200'}`}>
                          {row.entry.action}
                        </span>
                        <span class="px-2 py-1 rounded bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-300">
                          {row.entry.rule_name}
                        </span>
                      </div>
                    </div>
                  {/if}
                {/each}
              {/if}
            </div>
          </div>

          <div class="col-span-1">
            <div class="p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 h-full space-y-3">
              <h3 class="text-lg font-semibold">Details</h3>
              {#if selectedPath}
                {#if currentEntry}
                  <div class="space-y-2">
                    <div class="flex items-center justify-between">
                      <div class="font-medium break-all">{currentEntry.path}</div>
                      {#if isProtectedPath(currentEntry.path)}
                        <span class="px-2 py-1 text-[10px] rounded bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-200">
                          Protected
                        </span>
                      {/if}
                    </div>
                    <div class="text-xs text-gray-500 dark:text-gray-400">
                      Rule: {currentEntry.rule_name} · {currentEntry.reason}
                    </div>
                    <div class="text-xs text-gray-500 dark:text-gray-400">
                      Size: {formatBytes(currentEntry.size)} · Modified: {currentEntry.modified}
                    </div>
                    <div class={`p-2 rounded text-sm ${currentEntry.action === 'delete'
                      ? 'bg-red-50 text-red-700 dark:bg-red-900/20 dark:text-red-100'
                      : currentEntry.action === 'keep'
                      ? 'bg-green-50 text-green-700 dark:bg-green-900/20 dark:text-green-100'
                      : 'bg-yellow-50 text-yellow-700 dark:bg-yellow-900/20 dark:text-yellow-100'}`}>
                      {statusBanner(currentEntry.action)}
                    </div>
                    <div class="space-y-2">
                      <div class="text-xs text-gray-600 dark:text-gray-400">Action</div>
                      <div class="flex gap-2">
                        <button class="btn-secondary text-xs" on:click={() => updateAction(currentEntry.path, 'keep')}>
                          Keep
                        </button>
                        <button class="btn-secondary text-xs" on:click={() => updateAction(currentEntry.path, 'review')}>
                          Review
                        </button>
                        <button
                          class="btn-primary text-xs disabled:opacity-50"
                          disabled={isProtectedPath(currentEntry.path)}
                          on:click={() => updateAction(currentEntry.path, 'delete')}
                        >
                          Delete
                        </button>
                      </div>
                    </div>
                  </div>
                {:else if currentDir}
                  <div class="space-y-2">
                    <div class="font-medium break-all">{currentDir.path}</div>
                    <div class="text-xs text-gray-500 dark:text-gray-400">
                      {currentDir.deleteCount} delete · {currentDir.reviewCount} review · {currentDir.keepCount} keep
                    </div>
                    <div class="text-xs text-gray-500 dark:text-gray-400">
                      Size: {formatBytes(currentDir.size)}
                    </div>
                    <div class="space-y-2">
                      <div class="text-xs text-gray-600 dark:text-gray-400">Apply to subtree</div>
                      <div class="flex flex-wrap gap-2">
                        <button class="btn-secondary text-xs" on:click={() => updateSubtreeAction(currentDir.path, 'keep')}>
                          Mark subtree keep
                        </button>
                        <button class="btn-secondary text-xs" on:click={() => updateSubtreeAction(currentDir.path, 'review')}>
                          Mark subtree review
                        </button>
                        <button
                          class="btn-primary text-xs"
                          on:click={() => updateSubtreeAction(currentDir.path, 'delete')}
                        >
                          Mark subtree delete
                        </button>
                      </div>
                      <p class="text-[11px] text-gray-500 dark:text-gray-400">
                        Protected paths within the subtree remain non-deletable.
                      </p>
                    </div>
                  </div>
                {:else}
                  <div class="text-gray-500 dark:text-gray-400 text-sm">
                    Select an item to see details. If filters are active, clear them to show hidden items.
                  </div>
                {/if}
              {:else}
                <div class="text-gray-500 dark:text-gray-400 text-sm">Select an item to see details.</div>
              {/if}
            </div>
          </div>
        </div>
      {:else}
        <div class="mt-4">
          <div class="p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 text-sm space-y-2 max-h-96 overflow-auto">
            {#if visibleEntries.length === 0}
              <div class="text-gray-500 dark:text-gray-400">No entries match the current filters.</div>
            {:else}
              {#each visibleEntries as entry}
                <div class="border-b border-gray-200 dark:border-gray-700 pb-2 mb-2 last:border-0 last:pb-0 last:mb-0">
                  <div class="flex items-center justify-between">
                    <div class="font-medium">{entry.path}</div>
                    <div class="flex gap-2 text-[10px]">
                      <span class={`px-2 py-1 rounded uppercase tracking-wide ${entry.action === 'delete'
                        ? 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-200'
                        : entry.action === 'review'
                        ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-200'
                        : 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-200'}`}>
                        {entry.action}
                      </span>
                      <button class="btn-secondary text-[10px]" on:click={() => focusPath(entry.path)}>
                        Focus in tree
                      </button>
                    </div>
                  </div>
                  <div class="text-xs text-gray-500 dark:text-gray-400 flex gap-2 flex-wrap">
                    <span>{entry.rule_name}</span>
                    <span>{entry.reason}</span>
                    <span>{formatBytes(entry.size)}</span>
                  </div>
                  <div class="flex gap-2 mt-2">
                    <button class="btn-secondary text-xs" on:click={() => updateAction(entry.path, 'keep')}>
                      Keep
                    </button>
                    <button class="btn-secondary text-xs" on:click={() => updateAction(entry.path, 'review')}>
                      Review
                    </button>
                    <button
                      class="btn-primary text-xs disabled:opacity-50"
                      disabled={isProtectedPath(entry.path)}
                      on:click={() => updateAction(entry.path, 'delete')}
                    >
                      Delete
                    </button>
                  </div>
                </div>
              {/each}
            {/if}
          </div>
        </div>
      {/if}
    {:else}
      <div class="p-8 bg-gray-50 dark:bg-gray-700 rounded-lg text-center">
        <p class="text-gray-500 dark:text-gray-400">
          No cleanup plan loaded. Generate a plan from the Scan page to populate this view.
        </p>
        <button class="btn-secondary mt-3" on:click={handleLoadPlan} disabled={loading}>
          {loading ? 'Loading...' : 'Load Plan'}
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

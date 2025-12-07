<script lang="ts">
  import { scanStore } from '../stores/scan';
  import type { CleanupEntry } from '../services/tauri';
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

  function formatBytes(bytes: number): string {
    if (!bytes) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    const value = bytes / Math.pow(1024, i);
    return `${value.toFixed(1)} ${units[i]}`;
  }

  function normalizePath(path: string): string {
    return path.replace(/\\/g, '/');
  }

  function buildTree(entries: CleanupEntry[]): TreeNode {
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

      // build directory chain
      for (const part of parts.slice(0, -1)) {
        accumulated = accumulated ? `${accumulated}/${part}` : part;
        const key = accumulated;
        let child = nodeMap.get(key);
        if (!child) {
          child = {
            name: part,
            path: key,
            depth: current.depth + 1,
            children: [],
            entries: [],
            size: 0,
            deleteCount: 0,
            reviewCount: 0,
            keepCount: 0,
          };
          current.children.push(child);
          nodeMap.set(key, child);
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

    return compute(root);
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

  $: stats = $scanStore.planStats;
  $: scan = $scanStore.scanResult;
  $: entries = ($scanStore.plan?.entries ?? []).filter((entry) => entry.path !== '.');
  let actionFilter: 'all' | 'delete' | 'review' | 'keep' = 'all';
  let search = '';
  let showActions = true;
  let expandedNodes: Set<string> = new Set(['__root__']);

  $: filteredEntries = entries.filter((entry) => {
    if (actionFilter !== 'all' && entry.action !== actionFilter) return false;
    if (search && !entry.path.toLowerCase().includes(search.toLowerCase())) return false;
    return true;
  });

  $: tree = buildTree(filteredEntries);
  $: treeRows = flattenTree(tree, expandedNodes);

  function toggleNode(path: string) {
    const next = new Set(expandedNodes);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    expandedNodes = next;
  }
</script>

<div class="container mx-auto p-8">
  <div class="card space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <p class="text-sm uppercase tracking-wide text-primary-600 font-semibold">Phase 4.3 preview</p>
        <h1 class="text-3xl font-bold">Scan Results</h1>
        <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
          Basic readout from the latest scan and plan with a visual tree.
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

        <div class="flex items-center gap-3 text-sm">
          <label class="text-gray-600 dark:text-gray-400" for="results-filter">Filter:</label>
          <select
            id="results-filter"
            class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700"
            bind:value={actionFilter}
          >
            <option value="all">All</option>
            <option value="delete">Delete</option>
            <option value="review">Review</option>
            <option value="keep">Keep</option>
          </select>
          <input
            class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 flex-1"
            placeholder="Search path."
            bind:value={search}
            type="text"
          />
          <button class="btn-secondary text-xs" on:click={() => (showActions = !showActions)}>
            {showActions ? 'Hide Actions' : 'Show Actions'}
          </button>
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
            </div>
          {/each}
          {#if filteredEntries.length > 10}
            <div class="text-xs text-gray-500 dark:text-gray-400">
              .and {filteredEntries.length - 10} more (scroll or refine filters)
            </div>
          {/if}
        </div>

        <div>
          <h3 class="text-lg font-semibold mt-4 mb-2">Tree View (Phase 4.3)</h3>
          <div class="p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 max-h-96 overflow-auto text-sm space-y-2">
            {#if treeRows.length === 0}
              <div class="text-gray-500 dark:text-gray-400">No entries to display. Run a scan and generate a plan.</div>
            {:else}
              {#each treeRows as row}
                {#if row.kind === 'dir'}
                  <div
                    class="flex items-center justify-between py-1 px-2 rounded hover:bg-gray-50 dark:hover:bg-gray-700"
                    style={`margin-left: ${row.depth * 12}px`}
                  >
                    <div class="flex items-center gap-2">
                      <button
                        class="text-xs px-2 py-1 rounded bg-gray-100 dark:bg-gray-700"
                        on:click={() => toggleNode(row.path)}
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
                    class="flex items-center justify-between py-1 px-2 rounded border border-gray-100 dark:border-gray-700 bg-gray-50 dark:bg-gray-900/40"
                    style={`margin-left: ${row.depth * 12}px`}
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
                      <span class="px-2 py-1 rounded bg-gray-200 dark:bg-gray-700 uppercase tracking-wide">{row.entry.action}</span>
                      {#if showActions}
                        <span class="px-2 py-1 rounded bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-300">
                          {row.entry.rule_name}
                        </span>
                      {/if}
                    </div>
                  </div>
                {/if}
              {/each}
            {/if}
          </div>
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

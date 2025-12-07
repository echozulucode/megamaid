<script lang="ts">
  import { onMount } from 'svelte';
  import {
    detectCleanupCandidates,
    generateCleanupPlan,
    getLastScanResult,
    getPlanStats,
    detectTauriRuntime,
    pickDirectory,
    scanDirectory,
    type DetectorConfig,
    type PlanConfig,
  } from '../services/tauri';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { scanStore } from '../stores/scan';

  let directory = '';
  let maxDepth: number | null = 5;
  let skipHidden = true;
  let followSymlinks = false;
  let threadCount = 0;
  let largeFileThresholdMb = 100;

  let scanning = false;
  let error: string | null = null;
  let tauriAvailable = false;
  let eventMessage = '';
  let unsubscribers: UnlistenFn[] = [];
  let filesScanned = 0;
  let totalFilesEstimate: number | null = null;
  let lastProgressPath: string | undefined = undefined;

  onMount(() => {
    let disposed = false;

    (async () => {
      tauriAvailable = await detectTauriRuntime();
      if (!tauriAvailable || disposed) return;

      listen<string>('scan:started', (event) => {
        eventMessage = `Scan started: ${event.payload}`;
        filesScanned = 0;
        scanStore.update((s) => ({ ...s, status: 'scanning' }));
      }).then((unsub: UnlistenFn) => unsubscribers.push(unsub));

      listen<{ path?: string; files_scanned?: number }>('scan:progress', (event) => {
        const payload = event.payload;
        if (typeof payload?.files_scanned === 'number') {
          filesScanned = payload.files_scanned;
        }
        lastProgressPath = payload?.path;
        scanStore.update((s) => ({
          ...s,
          status: 'scanning',
          filesScanned,
          lastProgressPath,
        }));
      }).then((unsub: UnlistenFn) => unsubscribers.push(unsub));

      listen<{ total_files?: number; total_size?: number; path?: string }>('scan:complete', (event) => {
        const payload = event.payload;
        const pathText = payload?.path ? ` (${payload.path})` : '';
        eventMessage = `Scan complete${pathText}`;
        filesScanned = payload?.total_files ?? filesScanned;
        totalFilesEstimate = payload?.total_files ?? null;
      }).then((unsub: UnlistenFn) => unsubscribers.push(unsub));

      listen<string>('scan:error', (event) => {
        const message = event.payload ?? 'Unknown scan error';
        error = message;
        scanStore.update((s) => ({ ...s, status: 'error', error: message }));
      }).then((unsub: UnlistenFn) => unsubscribers.push(unsub));
    })();

    return () => {
      disposed = true;
      unsubscribers.forEach((u) => u());
    };
  });

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    const value = bytes / Math.pow(1024, i);
    return `${value.toFixed(1)} ${units[i]}`;
  }

  async function chooseDirectory() {
    error = null;
    try {
      const selected = await pickDirectory();
      if (selected) {
        directory = selected;
        tauriAvailable = true;
        scanStore.update((state) => ({ ...state, directory }));
      }
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }

  async function startScan() {
    error = null;

    if (!directory) {
      error = 'Select a directory to scan first.';
      return;
    }

    if (!tauriAvailable) {
      error = 'Scanning requires the desktop app (Tauri runtime not detected).';
      return;
    }

    scanning = true;
    scanStore.set({
      directory,
      status: 'scanning',
      scanResult: undefined,
      detections: undefined,
      plan: undefined,
      planStats: undefined,
      error: null,
      filesScanned: 0,
    });

    try {
      const result = await scanDirectory(directory, {
        max_depth: maxDepth ?? null,
        skip_hidden: skipHidden,
        follow_symlinks: followSymlinks,
        thread_count: threadCount,
      });

      const detectorConfig: DetectorConfig = {
        size_threshold_mb: largeFileThresholdMb ?? null,
        enable_build_artifacts: true,
      };

      const detections = await detectCleanupCandidates(result.entries, detectorConfig);

      const planConfig: PlanConfig = {
        base_path: directory,
        output_path: `${directory}/megamaid-plan.yaml`,
      };

      const plan = await generateCleanupPlan(detections, planConfig);
      const planStats = await getPlanStats(plan);

      scanStore.set({
        directory,
        status: 'planned',
        scanResult: result,
        detections,
        plan,
        planStats,
        error: null,
        filesScanned,
      });
      totalFilesEstimate = result.total_files;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      error = message;
      scanStore.set({
        directory,
        status: 'error',
        error: message,
        scanResult: undefined,
        detections: undefined,
        plan: undefined,
        planStats: undefined,
        filesScanned,
      });
    } finally {
      scanning = false;
    }
  }

  async function refreshLastScan() {
    try {
      const last = await getLastScanResult();
      if (last) {
        scanStore.update((s) => ({
          ...s,
          scanResult: last,
          directory: directory || s.directory,
        }));
        eventMessage = 'Loaded last scan from backend state.';
      } else {
        eventMessage = 'No previous scan found.';
      }
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }
</script>

<div class="container mx-auto p-8">
  <div class="card space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <p class="text-sm uppercase tracking-wide text-primary-600 font-semibold">Phase 4.1</p>
        <h1 class="text-3xl font-bold">Scan Directory</h1>
        <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
          Wire up the core scan flow so the GUI can drive the Rust engine.
        </p>
      </div>
      <div class="text-xs text-gray-500 dark:text-gray-400">
        {tauriAvailable ? 'Tauri runtime detected' : 'Dev preview (no Tauri runtime)'}
      </div>
    </div>

    <div>
      <label class="block text-sm font-medium mb-2" for="scan-directory">
        Select Directory
      </label>
      <div class="flex gap-2">
        <input
          id="scan-directory"
          type="text"
          placeholder="Choose a directory to scan..."
          class="flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700"
          readonly
          bind:value={directory}
        />
        <button class="btn-secondary" on:click|preventDefault={chooseDirectory}>
          Browse
        </button>
      </div>
    </div>

    <div>
      <p class="block text-sm font-medium mb-2">
        Scan Options
      </p>
      <div class="grid grid-cols-2 gap-4">
        <div>
          <label class="block text-xs text-gray-600 dark:text-gray-400 mb-1" for="scan-max-depth">
            Max Depth (empty for unlimited)
          </label>
          <input
            id="scan-max-depth"
            type="number"
            min="1"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700"
            bind:value={maxDepth}
            placeholder="Unlimited"
          />
        </div>
        <div>
          <label class="block text-xs text-gray-600 dark:text-gray-400 mb-1" for="scan-threshold">
            Large File Threshold (MB)
          </label>
          <input
            id="scan-threshold"
            type="number"
            min="1"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700"
            bind:value={largeFileThresholdMb}
          />
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Used for detector sizing; defaults to 100MB.
          </p>
        </div>
        <div>
          <label class="block text-xs text-gray-600 dark:text-gray-400 mb-1" for="scan-thread-count">
            Thread Count (0 = auto)
          </label>
          <input
            id="scan-thread-count"
            type="number"
            min="0"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700"
            bind:value={threadCount}
          />
        </div>
        <div>
          <label class="block text-xs text-gray-600 dark:text-gray-400 mb-1" for="scan-follow-symlinks">
            Follow Symlinks
          </label>
          <label class="flex items-center gap-2" for="scan-follow-symlinks">
            <input id="scan-follow-symlinks" type="checkbox" class="rounded" bind:checked={followSymlinks} />
            <span class="text-sm">Enable symbolic link traversal</span>
          </label>
        </div>
      </div>

      <div class="mt-4 flex items-center gap-2">
        <input id="skipHidden" type="checkbox" class="rounded" bind:checked={skipHidden} />
        <label for="skipHidden" class="text-sm">Skip hidden files and directories</label>
      </div>
    </div>

    <div class="flex gap-3">
      <button class="btn-primary disabled:opacity-60" disabled={scanning} on:click|preventDefault={startScan}>
        {scanning ? 'Scanning…' : 'Start Scan'}
      </button>
      <button class="btn-secondary" disabled>
        Cancel
      </button>
      <button class="btn-secondary" on:click|preventDefault={refreshLastScan}>
        Load Last Scan
      </button>
    </div>

    {#if error}
      <div class="p-3 border border-red-200 dark:border-red-700 bg-red-50 dark:bg-red-900/30 rounded-lg text-sm text-red-800 dark:text-red-100">
        {error}
      </div>
    {/if}

    {#if eventMessage}
      <div class="p-3 border border-blue-200 dark:border-blue-700 bg-blue-50 dark:bg-blue-900/30 rounded-lg text-sm text-blue-800 dark:text-blue-100">
        {eventMessage}
        {#if filesScanned > 0}
          <span class="ml-2 text-xs text-gray-600 dark:text-gray-300">
            {filesScanned.toLocaleString()} files scanned
          </span>
        {/if}
      </div>
    {/if}

    {#if $scanStore.planStats}
      <div class="p-4 border border-gray-200 dark:border-gray-700 rounded-lg bg-gray-50 dark:bg-gray-800/60 space-y-2">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm text-gray-600 dark:text-gray-400">Scan + Plan Complete</p>
            <p class="text-xl font-semibold mt-1">{$scanStore.planStats.total_entries.toLocaleString()} flagged items</p>
            <p class="text-sm text-gray-600 dark:text-gray-400">
              {formatBytes($scanStore.planStats.total_size)} total across {($scanStore.scanResult?.total_files ?? 0).toLocaleString()} entries
            </p>
          </div>
          <div class="text-xs text-gray-500 dark:text-gray-400">
            {$scanStore.planStats.delete_count} delete · {$scanStore.planStats.review_count} review · {$scanStore.planStats.keep_count} keep
          </div>
        </div>
      </div>
    {:else if !tauriAvailable}
      <div class="p-3 border border-yellow-200 dark:border-yellow-800 bg-yellow-50 dark:bg-yellow-900/20 rounded-lg text-sm text-yellow-800 dark:text-yellow-100">
        Launch the packaged desktop app to run real scans. UI can still be previewed in the browser.
      </div>
    {:else if scanning}
      <div class="p-4 border border-gray-200 dark:border-gray-700 rounded-lg bg-blue-50 dark:bg-blue-900/30 text-sm text-blue-900 dark:text-blue-100 space-y-1">
        <div class="flex items-center justify-between">
          <span class="font-semibold">Scanning…</span>
          <span>{filesScanned.toLocaleString()} files</span>
        </div>
        {#if lastProgressPath}
          <div class="text-xs text-blue-900/80 dark:text-blue-100/80 truncate">
            {lastProgressPath}
          </div>
        {/if}
        {#if totalFilesEstimate}
          <div class="h-2 bg-blue-100 dark:bg-blue-800 rounded">
            <div
              class="h-2 bg-blue-500 rounded"
              style={`width: ${Math.min(100, (filesScanned / totalFilesEstimate) * 100).toFixed(1)}%`}
            ></div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

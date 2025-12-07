import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';

export type ScannerConfig = {
  max_depth: number | null;
  skip_hidden: boolean;
  follow_symlinks: boolean;
  thread_count: number;
};

export type FileEntry = {
  path: string;
  size: number;
  modified: number;
  entry_type: 'File' | 'Directory';
  file_id?: number | null;
};

export type DetectionResult = {
  entry: FileEntry;
  rule_name: string;
  reason: string;
};

export type CleanupAction = 'delete' | 'keep' | 'review';

export type CleanupEntry = {
  path: string;
  size: number;
  modified: string;
  action: CleanupAction;
  rule_name: string;
  reason: string;
};

export type CleanupPlan = {
  version: string;
  created_at: string;
  base_path: string;
  entries: CleanupEntry[];
};

export type PlanStats = {
  total_entries: number;
  delete_count: number;
  review_count: number;
  keep_count: number;
  total_size: number;
};

export type DetectorConfig = {
  size_threshold_mb: number | null;
  enable_build_artifacts: boolean;
};

export type PlanConfig = {
  base_path: string;
  output_path: string;
};

export type PlanFileDialogOptions = {
  defaultPath?: string;
};

export type ScanResult = {
  entries: FileEntry[];
  total_files: number;
  total_size: number;
  errors: string[];
};

declare global {
  interface Window {
    __TAURI__?: unknown;
  }
}

export const isTauriSync = (): boolean =>
  typeof window !== 'undefined' && Boolean((window as any).__TAURI__);

export async function detectTauriRuntime(): Promise<boolean> {
  if (isTauriSync()) return true;
  try {
    // If the API loads, we're in Tauri.
    await import('@tauri-apps/api/core');
    return true;
  } catch {
    return false;
  }
}

async function ensureTauri() {
  if (await detectTauriRuntime()) return;
  throw new Error('Tauri runtime is not available. Launch the desktop app.');
}

export async function pickDirectory(): Promise<string | null> {
  await ensureTauri();
  const selection = await open({
    directory: true,
    multiple: false,
  });

  if (Array.isArray(selection)) {
    return selection[0] ?? null;
  }

  return selection ?? null;
}

export async function scanDirectory(
  path: string,
  config: ScannerConfig
): Promise<ScanResult> {
  await ensureTauri();

  return invoke<ScanResult>('scan_directory', { path, config });
}

export async function detectCleanupCandidates(
  entries: FileEntry[],
  config: DetectorConfig
): Promise<DetectionResult[]> {
  await ensureTauri();

  return invoke<DetectionResult[]>('detect_cleanup_candidates', { entries, config });
}

export async function generateCleanupPlan(
  detections: DetectionResult[],
  config: PlanConfig
): Promise<CleanupPlan> {
  await ensureTauri();

  return invoke<CleanupPlan>('generate_cleanup_plan', { detections, config });
}

export async function getPlanStats(plan: CleanupPlan): Promise<PlanStats> {
  await ensureTauri();

  return invoke<PlanStats>('get_plan_stats', { plan });
}

export async function getLastScanResult(): Promise<ScanResult | null> {
  await ensureTauri();

  const result = await invoke<ScanResult | null>('get_scan_results');
  return result;
}

export async function loadPlanFromFile(): Promise<CleanupPlan | null> {
  await ensureTauri();
  const path = await open({
    directory: false,
    multiple: false,
    filters: [{ name: 'Plans', extensions: ['yaml', 'yml', 'toml'] }],
  });
  const selected = Array.isArray(path) ? path[0] : path;
  if (!selected) return null;
  return invoke<CleanupPlan>('load_cleanup_plan', { path: selected });
}

export async function savePlanToFile(plan: CleanupPlan): Promise<string | null> {
  await ensureTauri();
  const baseDir = plan.base_path ? plan.base_path.toString() : undefined;
  const defaultPath = baseDir
    ? `${baseDir.replace(/[/\\\\]$/, '')}/megamaid-plan.yaml`
    : 'megamaid-plan.yaml';
  const target = await save({
    filters: [{ name: 'Plans', extensions: ['yaml', 'yml', 'toml'] }],
    defaultPath,
  });
  if (!target) {
    return null;
  }
  await invoke<void>('save_cleanup_plan', { plan, output_path: target });
  return target;
}

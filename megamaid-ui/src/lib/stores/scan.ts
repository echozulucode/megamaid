import { writable } from 'svelte/store';
import type {
  CleanupPlan,
  DetectionResult,
  PlanStats,
  ScanResult,
} from '../services/tauri';

export type ScanStatus = 'idle' | 'scanning' | 'detected' | 'planned' | 'error';

export type ScanState = {
  directory: string;
  status: ScanStatus;
  scanResult?: ScanResult;
  detections?: DetectionResult[];
  plan?: CleanupPlan;
  planStats?: PlanStats;
  error?: string | null;
  filesScanned: number;
  lastProgressPath?: string;
};

function sanitizePlan(plan?: CleanupPlan): CleanupPlan | undefined {
  if (!plan) return plan;
  const entries = (plan.entries ?? []).filter((entry) => entry.path !== '.');
  return { ...plan, entries };
}

function recomputePlanStats(plan?: CleanupPlan): PlanStats | undefined {
  if (!plan) return undefined;
  const delete_count = plan.entries.filter((e) => e.action === 'delete').length;
  const review_count = plan.entries.filter((e) => e.action === 'review').length;
  const keep_count = plan.entries.filter((e) => e.action === 'keep').length;
  const total_size = plan.entries.reduce((acc, e) => acc + e.size, 0);
  return {
    total_entries: plan.entries.length,
    delete_count,
    review_count,
    keep_count,
    total_size,
  };
}

function loadInitialState(): ScanState {
  if (typeof localStorage !== 'undefined') {
    const raw = localStorage.getItem('megamaid-scan-state');
    if (raw) {
      try {
        const parsed = JSON.parse(raw) as Partial<ScanState>;
        const sanitizedPlan = sanitizePlan(parsed.plan);
        return {
          directory: parsed.directory ?? '',
          status: parsed.status ?? 'idle',
          scanResult: parsed.scanResult,
          detections: parsed.detections,
          plan: sanitizedPlan,
          planStats: recomputePlanStats(sanitizedPlan) ?? parsed.planStats,
          error: parsed.error ?? null,
          filesScanned: parsed.filesScanned ?? 0,
          lastProgressPath: parsed.lastProgressPath,
        };
      } catch {
        // fall through to default
      }
    }
  }
  return {
    directory: '',
    status: 'idle',
    error: null,
    filesScanned: 0,
  };
}

const initialState: ScanState = loadInitialState();

export const scanStore = writable<ScanState>(initialState);

scanStore.subscribe((state) => {
  if (typeof localStorage === 'undefined') return;
  const toPersist: Partial<ScanState> = {
    directory: state.directory,
    status: state.status,
    plan: state.plan,
    planStats: state.planStats,
    filesScanned: state.filesScanned,
    lastProgressPath: state.lastProgressPath,
  };
  localStorage.setItem('megamaid-scan-state', JSON.stringify(toPersist));
});

export function resetScanState() {
  if (typeof localStorage !== 'undefined') {
    localStorage.removeItem('megamaid-scan-state');
  }
  scanStore.set({
    directory: '',
    status: 'idle',
    error: null,
    filesScanned: 0,
  });
}

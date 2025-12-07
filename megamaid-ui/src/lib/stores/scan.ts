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

function loadInitialState(): ScanState {
  if (typeof localStorage !== 'undefined') {
    const raw = localStorage.getItem('megamaid-scan-state');
    if (raw) {
      try {
        const parsed = JSON.parse(raw) as Partial<ScanState>;
        return {
          directory: parsed.directory ?? '',
          status: parsed.status ?? 'idle',
          scanResult: parsed.scanResult,
          detections: parsed.detections,
          plan: parsed.plan,
          planStats: parsed.planStats,
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

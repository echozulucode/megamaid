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
};

const initialState: ScanState = {
  directory: '',
  status: 'idle',
  error: null,
};

export const scanStore = writable<ScanState>(initialState);

export function resetScanState() {
  scanStore.set(initialState);
}

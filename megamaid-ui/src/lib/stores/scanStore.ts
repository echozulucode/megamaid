import { writable, derived } from 'svelte/store';

export interface ScanConfig {
  path: string;
  maxDepth: number;
  skipHidden: boolean;
  largeFileThreshold: number; // in MB
}

export interface ScanProgress {
  filesScanned: number;
  directoriesScanned: number;
  currentPath: string;
  isScanning: boolean;
}

export interface ScanResult {
  totalFiles: number;
  totalSize: number;
  cleanupCandidates: number;
  potentialSpaceSaved: number;
  timestamp: Date;
}

// Default configuration
const defaultConfig: ScanConfig = {
  path: '',
  maxDepth: 10,
  skipHidden: true,
  largeFileThreshold: 100,
};

const defaultProgress: ScanProgress = {
  filesScanned: 0,
  directoriesScanned: 0,
  currentPath: '',
  isScanning: false,
};

const defaultResult: ScanResult = {
  totalFiles: 0,
  totalSize: 0,
  cleanupCandidates: 0,
  potentialSpaceSaved: 0,
  timestamp: new Date(),
};

// Stores
export const scanConfig = writable<ScanConfig>(defaultConfig);
export const scanProgress = writable<ScanProgress>(defaultProgress);
export const scanResult = writable<ScanResult>(defaultResult);

// Derived store for scan status
export const isScanActive = derived(
  scanProgress,
  ($progress) => $progress.isScanning
);

// Actions
export const scanActions = {
  updateConfig(config: Partial<ScanConfig>) {
    scanConfig.update((current) => ({ ...current, ...config }));
  },

  startScan() {
    scanProgress.update((p) => ({ ...p, isScanning: true, filesScanned: 0, directoriesScanned: 0 }));
  },

  updateProgress(progress: Partial<ScanProgress>) {
    scanProgress.update((current) => ({ ...current, ...progress }));
  },

  completeScan(result: ScanResult) {
    scanResult.set(result);
    scanProgress.update((p) => ({ ...p, isScanning: false }));
  },

  cancelScan() {
    scanProgress.update((p) => ({ ...p, isScanning: false }));
  },

  reset() {
    scanConfig.set(defaultConfig);
    scanProgress.set(defaultProgress);
    scanResult.set(defaultResult);
  },
};

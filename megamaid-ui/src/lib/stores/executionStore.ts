import { writable, derived } from 'svelte/store';

export type ExecutionMode = 'DryRun' | 'Interactive' | 'Backup' | 'RecycleBin' | 'Direct';

export interface ExecutionConfig {
  mode: ExecutionMode;
  verifyPlan: boolean;
  failFast: boolean;
  backupDir?: string;
}

export interface ExecutionProgress {
  isExecuting: boolean;
  totalOperations: number;
  completedOperations: number;
  successfulOperations: number;
  failedOperations: number;
  skippedOperations: number;
  currentOperation: string;
  spaceFreed: number;
}

export interface ExecutionLog {
  timestamp: Date;
  level: 'info' | 'success' | 'warning' | 'error';
  message: string;
  path?: string;
}

// Default configuration
const defaultConfig: ExecutionConfig = {
  mode: 'DryRun',
  verifyPlan: true,
  failFast: false,
};

const defaultProgress: ExecutionProgress = {
  isExecuting: false,
  totalOperations: 0,
  completedOperations: 0,
  successfulOperations: 0,
  failedOperations: 0,
  skippedOperations: 0,
  currentOperation: '',
  spaceFreed: 0,
};

// Stores
export const executionConfig = writable<ExecutionConfig>(defaultConfig);
export const executionProgress = writable<ExecutionProgress>(defaultProgress);
export const executionLogs = writable<ExecutionLog[]>([]);

// Derived stores
export const isExecuting = derived(
  executionProgress,
  ($progress) => $progress.isExecuting
);

export const executionPercentage = derived(executionProgress, ($progress) => {
  if ($progress.totalOperations === 0) return 0;
  return Math.round(($progress.completedOperations / $progress.totalOperations) * 100);
});

// Actions
export const executionActions = {
  updateConfig(config: Partial<ExecutionConfig>) {
    executionConfig.update((current) => ({ ...current, ...config }));
  },

  startExecution(totalOperations: number) {
    executionProgress.set({
      isExecuting: true,
      totalOperations,
      completedOperations: 0,
      successfulOperations: 0,
      failedOperations: 0,
      skippedOperations: 0,
      currentOperation: '',
      spaceFreed: 0,
    });
    executionLogs.set([]);
  },

  updateProgress(progress: Partial<ExecutionProgress>) {
    executionProgress.update((current) => ({ ...current, ...progress }));
  },

  addLog(log: Omit<ExecutionLog, 'timestamp'>) {
    executionLogs.update((logs) => [
      ...logs,
      { ...log, timestamp: new Date() },
    ]);
  },

  completeExecution() {
    executionProgress.update((p) => ({ ...p, isExecuting: false }));
  },

  cancelExecution() {
    executionProgress.update((p) => ({ ...p, isExecuting: false }));
    executionActions.addLog({
      level: 'warning',
      message: 'Execution cancelled by user',
    });
  },

  reset() {
    executionConfig.set(defaultConfig);
    executionProgress.set(defaultProgress);
    executionLogs.set([]);
  },
};

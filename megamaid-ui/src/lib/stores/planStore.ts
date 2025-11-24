import { writable, derived } from 'svelte/store';

export type CleanupAction = 'Delete' | 'Review' | 'Keep';

export interface CleanupEntry {
  path: string;
  size: number;
  modified: string;
  action: CleanupAction;
  reason?: string;
}

export interface CleanupPlan {
  version: string;
  basePath: string;
  created: string;
  entries: CleanupEntry[];
}

export interface PlanStats {
  totalEntries: number;
  deleteCount: number;
  reviewCount: number;
  keepCount: number;
  totalSize: number;
  deleteSize: number;
}

// Default plan
const defaultPlan: CleanupPlan | null = null;

// Store
export const cleanupPlan = writable<CleanupPlan | null>(defaultPlan);

// Derived stores
export const planStats = derived(cleanupPlan, ($plan): PlanStats => {
  if (!$plan) {
    return {
      totalEntries: 0,
      deleteCount: 0,
      reviewCount: 0,
      keepCount: 0,
      totalSize: 0,
      deleteSize: 0,
    };
  }

  const stats = $plan.entries.reduce(
    (acc, entry) => {
      acc.totalEntries++;
      acc.totalSize += entry.size;

      switch (entry.action) {
        case 'Delete':
          acc.deleteCount++;
          acc.deleteSize += entry.size;
          break;
        case 'Review':
          acc.reviewCount++;
          break;
        case 'Keep':
          acc.keepCount++;
          break;
      }

      return acc;
    },
    {
      totalEntries: 0,
      deleteCount: 0,
      reviewCount: 0,
      keepCount: 0,
      totalSize: 0,
      deleteSize: 0,
    }
  );

  return stats;
});

export const hasPlan = derived(cleanupPlan, ($plan) => $plan !== null);

// Actions
export const planActions = {
  loadPlan(plan: CleanupPlan) {
    cleanupPlan.set(plan);
  },

  updateEntry(index: number, updates: Partial<CleanupEntry>) {
    cleanupPlan.update((plan) => {
      if (!plan) return plan;

      const newEntries = [...plan.entries];
      newEntries[index] = { ...newEntries[index], ...updates };

      return { ...plan, entries: newEntries };
    });
  },

  updateEntryAction(index: number, action: CleanupAction) {
    planActions.updateEntry(index, { action });
  },

  batchUpdateAction(indices: number[], action: CleanupAction) {
    cleanupPlan.update((plan) => {
      if (!plan) return plan;

      const newEntries = [...plan.entries];
      indices.forEach((index) => {
        newEntries[index] = { ...newEntries[index], action };
      });

      return { ...plan, entries: newEntries };
    });
  },

  clear() {
    cleanupPlan.set(null);
  },
};

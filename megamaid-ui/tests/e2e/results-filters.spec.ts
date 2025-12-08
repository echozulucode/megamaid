import { test, expect } from '@playwright/test';

const seedPlanScript = `
(() => {
  const plan = {
    version: '1.0.0',
    created_at: '2025-01-01T00:00:00Z',
    base_path: 'C:/workspace',
    entries: [
      { path: 'project/node_modules', size: 2048, modified: '2025-01-01T00:00:00Z', action: 'delete', rule_name: 'build_artifact', reason: 'Junk dir' },
      { path: 'project/logs/app.log', size: 4096, modified: '2025-01-01T00:00:00Z', action: 'review', rule_name: 'large_file', reason: 'Large file' },
      { path: 'project/src/index.ts', size: 1024, modified: '2025-01-01T00:00:00Z', action: 'keep', rule_name: 'source', reason: 'Source code' }
    ]
  };
  const stats = {
    total_entries: plan.entries.length,
    delete_count: plan.entries.filter(e => e.action === 'delete').length,
    review_count: plan.entries.filter(e => e.action === 'review').length,
    keep_count: plan.entries.filter(e => e.action === 'keep').length,
    total_size: plan.entries.reduce((a, e) => a + e.size, 0),
  };
  const scanResult = {
    entries: [],
    total_files: 10,
    total_size: 7168,
    errors: []
  };
  const state = {
    directory: 'C:/workspace',
    status: 'planned',
    plan,
    planStats: stats,
    scanResult,
    filesScanned: 10,
  };
  window.localStorage.setItem('megamaid-scan-state', JSON.stringify(state));
})();
`;

test.describe('Results filters', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(seedPlanScript);
  });

  test('action filter and search narrow list', async ({ page }) => {
    await page.goto('/#/results');
    await expect(page.getByText('Cleanup Candidates')).toBeVisible();

    // Filter to delete
    await page.waitForSelector('#results-filter', { state: 'visible' });
    await page.locator('#results-filter').selectOption('delete');
    await expect(page.getByText('node_modules', { exact: false })).toBeVisible();
    await expect(page.getByText('app.log', { exact: false })).toHaveCount(0);
    await expect(page.getByText('index.ts', { exact: false })).toHaveCount(0);

    // Search further
    await page.getByPlaceholder('Search path.').fill('node_modules');
    await expect(page.getByText('node_modules', { exact: false })).toBeVisible();
  });
});

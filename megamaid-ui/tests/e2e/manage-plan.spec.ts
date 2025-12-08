import { test, expect } from '@playwright/test';

const seedPlanScript = `
(() => {
  const plan = {
    version: '1.0.0',
    created_at: '2025-01-01T00:00:00Z',
    base_path: 'C:/workspace',
    entries: [
      { path: 'project/.git', size: 1024, modified: '2025-01-01T00:00:00Z', action: 'keep', rule_name: 'protected', reason: 'VCS' },
      { path: 'project/node_modules', size: 2048, modified: '2025-01-01T00:00:00Z', action: 'delete', rule_name: 'build_artifact', reason: 'Junk dir' },
      { path: 'project/logs/app.log', size: 4096, modified: '2025-01-01T00:00:00Z', action: 'review', rule_name: 'large_file', reason: 'Large file' }
    ]
  };
  const stats = {
    total_entries: plan.entries.length,
    delete_count: plan.entries.filter(e => e.action === 'delete').length,
    review_count: plan.entries.filter(e => e.action === 'review').length,
    keep_count: plan.entries.filter(e => e.action === 'keep').length,
    total_size: plan.entries.reduce((a, e) => a + e.size, 0),
  };
  const state = {
    directory: 'C:/workspace',
    status: 'planned',
    plan,
    planStats: stats,
    filesScanned: 0,
  };
  window.localStorage.setItem('megamaid-scan-state', JSON.stringify(state));
})();
`;

test.describe('Manage Plan', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(seedPlanScript);
  });

  test('tree/list toggle and detail pane selection', async ({ page }) => {
    await page.goto('/#/plan');
    await expect(page.getByRole('heading', { name: 'Plan Review & Actions' })).toBeVisible();

    // Switch to list and focus a protected entry back into tree
    await page.getByRole('button', { name: 'List' }).click();
    const protectedRow = page.getByText('project/.git', { exact: false }).first();
    await expect(protectedRow).toBeVisible();
    await protectedRow.locator('..').getByRole('button', { name: 'Focus in tree' }).click();

    // Detail pane shows selection and protected badge; delete disabled
    await expect(page.getByText('project/.git', { exact: false })).toBeVisible();
    const detailPanel = page.getByRole('heading', { name: 'Details' }).locator('xpath=../..');
    const deleteBtn = detailPanel.getByRole('button', { name: 'Delete', exact: true });
    await expect(deleteBtn).toBeDisabled();
  });

  test('pending deletes filter reduces entries', async ({ page }) => {
    await page.goto('/#/plan');
    // Switch to list for simpler assertions
    await page.getByRole('button', { name: 'List' }).click();
    await expect(page.getByText('node_modules', { exact: false })).toBeVisible();

    // Enable pending deletes
    await page.getByLabel('Pending deletes only').check();
    await expect(page.getByText('node_modules', { exact: false })).toBeVisible();
    // Non-delete entries hidden
    await expect(page.getByText('.git', { exact: false })).toHaveCount(0);
    await expect(page.getByText('app.log', { exact: false })).toHaveCount(0);
  });
});

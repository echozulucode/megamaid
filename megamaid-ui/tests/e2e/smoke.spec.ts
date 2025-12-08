import { test, expect } from '@playwright/test';

test('app renders and shows nav', async ({ page }) => {
  await page.goto('/');

  const sidebar = page.getByRole('navigation');
  await expect(sidebar.getByRole('heading', { name: 'Megamaid' })).toBeVisible();
  await expect(sidebar.getByRole('link', { name: 'Scan' })).toBeVisible();
});

test('scan form inputs are present', async ({ page }) => {
  await page.goto('/#/scan');
  await expect(page.getByLabel('Select Directory')).toBeVisible();
  await expect(page.getByLabel('Max Depth (empty for unlimited)')).toBeVisible();
  await expect(page.getByLabel('Large File Threshold (MB)')).toBeVisible();
});

test('progress/status area renders', async ({ page }) => {
  await page.goto('/#/scan');
  await expect(page.getByRole('heading', { name: 'Scan Directory' })).toBeVisible();
});

test('results summary cards render', async ({ page }) => {
  await page.goto('/#/results');
  await expect(page.getByText('Files Scanned')).toBeVisible();
  await expect(page.getByText('Cleanup Candidates')).toBeVisible();
});

test('plan summary renders', async ({ page }) => {
  await page.goto('/#/plan');
  await expect(page.getByRole('heading', { name: 'Plan Review & Actions' })).toBeVisible();
  await expect(page.getByText('No cleanup plan loaded.', { exact: false })).toBeVisible();
});

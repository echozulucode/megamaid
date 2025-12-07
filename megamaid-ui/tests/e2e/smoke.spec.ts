import { test, expect } from '@playwright/test';

test('app renders and shows nav', async ({ page }) => {
  await page.goto('/');

  const sidebar = page.getByRole('navigation');
  await expect(sidebar.getByRole('heading', { name: 'Megamaid' })).toBeVisible();
  await expect(sidebar.getByRole('link', { name: 'Scan' })).toBeVisible();
});

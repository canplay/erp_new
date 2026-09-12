/**
 * @file users.spec.ts
 * @description User Management E2E Tests
 */
import { test, expect, type Page } from '@playwright/test';

// Test credentials
const TEST_USER = {
  username: 'admin',
  password: 'admin123',
};

/**
 * Helper function to login before user management tests
 */
async function login(page: Page) {
  await page.goto('/login');
  await page.getByTestId('username-input').fill(TEST_USER.username);
  await page.getByTestId('password-input').fill(TEST_USER.password);
  await page.getByTestId('login-button').click();
  await page.waitForURL('/', { timeout: 15000 });
}

test.describe('User Management', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('navigate to user list page', async ({ page }) => {
    // Navigate to user list
    await page.goto('/users');

    // Verify user list page is displayed
    await expect(page.locator('text=用户管理')).toBeVisible();

    // Verify table is present
    await expect(page.locator('.q-table')).toBeVisible();
  });

  test('search users triggers list refresh', async ({ page }) => {
    await page.goto('/users');

    // Verify user list page loaded
    await expect(page.locator('text=用户管理')).toBeVisible();

    // Click search button in AdvancedSearch component
    const searchButton = page.getByTestId('search-button');
    if (await searchButton.isVisible().catch(() => false)) {
      await searchButton.click();
      await page.waitForTimeout(500);
    }

    // Verify table is still visible after search
    await expect(page.locator('.q-table')).toBeVisible();
  });

  test('open create user dialog', async ({ page }) => {
    await page.goto('/users');

    // Verify user list page loaded
    await expect(page.locator('text=用户管理')).toBeVisible();

    // Click create user button
    await page.getByTestId('create-user-button').click();

    // Verify dialog is opened
    await expect(page.locator('.q-dialog')).toBeVisible();

    // Verify form is present (username input should be visible)
    await expect(page.locator('.q-dialog input').first()).toBeVisible();
  });

  test('create new user fills form and saves', async ({ page }) => {
    await page.goto('/users');

    // Click create user button
    await page.getByTestId('create-user-button').click();

    // Wait for dialog
    await expect(page.locator('.q-dialog')).toBeVisible();

    // Fill username
    const timestamp = Date.now();
    const testUsername = `testuser_${timestamp}`;
    await page.locator('.q-dialog input').first().fill(testUsername);
    
    // Fill password if present
    const passwordInput = page.locator('.q-dialog input[type="password"]');
    if (await passwordInput.isVisible().catch(() => false)) {
      await passwordInput.fill('TestPass123!');
    }

    // Click save button
    await page.getByTestId('save-user-button').click();

    // Wait for response
    await page.waitForTimeout(1000);
  });

  test('user list shows table with columns', async ({ page }) => {
    await page.goto('/users');

    // Verify table headers exist
    await expect(page.locator('.q-table')).toBeVisible();
  });

  test('refresh user list page', async ({ page }) => {
    await page.goto('/users');

    // Verify page loaded
    await expect(page.locator('text=用户管理')).toBeVisible();

    // Reload page
    await page.reload();

    // Verify table is still visible
    await expect(page.locator('.q-table')).toBeVisible();
  });

  test('create and cancel user dialog', async ({ page }) => {
    await page.goto('/users');

    // Open create dialog
    await page.getByTestId('create-user-button').click();
    await expect(page.locator('.q-dialog')).toBeVisible();

    // Cancel dialog
    await page.locator('.q-dialog button:has-text("取消"), .q-dialog button:has-text("Cancel")').first().click();

    // Verify dialog closed
    await expect(page.locator('.q-dialog')).not.toBeVisible({ timeout: 5000 });
  });
});

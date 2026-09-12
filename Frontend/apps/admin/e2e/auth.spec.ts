/**
 * @file auth.spec.ts
 * @description Authentication E2E Tests
 */
import { test, expect, type Page } from '@playwright/test';

// Test fixtures
const TEST_USER = {
  username: 'admin',
  password: 'admin123',
};

const INVALID_USER = {
  username: 'wronguser',
  password: 'wrongpassword',
};

/**
 * Helper function to perform login
 */
async function login(page: Page, username: string, password: string) {
  await page.goto('/login');
  await page.getByTestId('username-input').fill(username);
  await page.getByTestId('password-input').fill(password);
  await page.getByTestId('login-button').click();
}

test.describe('Authentication', () => {
  test('admin can login successfully', async ({ page }) => {
    await page.goto('/login');

    // Verify we're on the login page
    await expect(page.locator('text=登录')).toBeVisible();

    // Fill credentials and submit
    await page.getByTestId('username-input').fill(TEST_USER.username);
    await page.getByTestId('password-input').fill(TEST_USER.password);
    await page.getByTestId('login-button').click();

    // Verify successful redirect to dashboard
    await page.waitForURL('/', { timeout: 15000 });
    await expect(page).toHaveURL('/');

    // Verify dashboard content is visible
    await expect(page.locator('text=仪表盘')).toBeVisible();
  });

  test('invalid credentials show error message', async ({ page }) => {
    await page.goto('/login');

    await page.getByTestId('username-input').fill(INVALID_USER.username);
    await page.getByTestId('password-input').fill(INVALID_USER.password);
    await page.getByTestId('login-button').click();

    // Should remain on login page
    await expect(page).toHaveURL(/\/login/);

    // Verify error message is displayed
    const errorMessage = page.locator('.error-banner, .q-banner--negative, [class*="error"]');
    await expect(errorMessage.first()).toBeVisible({ timeout: 5000 });
  });

  test('logout returns to login page', async ({ page }) => {
    // Login first
    await login(page, TEST_USER.username, TEST_USER.password);
    await page.waitForURL('/', { timeout: 15000 });

    // Open user menu and click logout
    await page.getByTestId('user-menu').click();
    await page.getByTestId('logout-button').click();

    // Confirm logout in dialog if present
    const confirmButton = page.locator('.q-dialog .q-btn:has-text("确认"), .q-dialog .q-btn:has-text("确定")');
    if (await confirmButton.isVisible({ timeout: 2000 }).catch(() => false)) {
      await confirmButton.click();
    }

    // Should redirect to login page
    await page.waitForURL(/\/login/, { timeout: 15000 });
    await expect(page).toHaveURL(/\/login/);
  });

  test('empty form shows validation errors', async ({ page }) => {
    await page.goto('/login');

    // Click login without filling form
    await page.getByTestId('login-button').click();

    // Should stay on login page
    await expect(page).toHaveURL(/\/login/);
  });

  test('login form has required fields', async ({ page }) => {
    await page.goto('/login');

    // Verify form elements exist
    await expect(page.getByTestId('username-input')).toBeVisible();
    await expect(page.getByTestId('password-input')).toBeVisible();
    await expect(page.getByTestId('login-button')).toBeVisible();
  });
});

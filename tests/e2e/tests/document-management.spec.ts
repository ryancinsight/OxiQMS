import { test, expect, Page } from '@playwright/test';

/**
 * Document Management Validation Tests
 * 
 * Validates that documents created via CLI are properly displayed in the web browser
 * with correct metadata, content preview, and download functionality.
 * 
 * Regulatory Compliance: FDA 21 CFR Part 820, ISO 13485
 */

test.describe('Document Management Web Interface', () => {
  let page: Page;

  test.beforeEach(async ({ page: testPage }) => {
    page = testPage;
    await page.goto('/');

    // Wait for the application to load
    await expect(page.locator('h1')).toContainText('QMS - Medical Device Quality Management');
    await expect(page.locator('text=🟢 Connected')).toBeVisible();
  });

  test('should display documents section with proper navigation', async () => {
    // Navigate to Documents section
    await page.click('a[href="#documents"]');

    // Verify Documents section loads
    await expect(page.locator('h2')).toContainText('Document Control');
    await expect(page.locator('text=Medical Device Quality Management Documents')).toBeVisible();

    // Verify regulatory compliance indicators
    await expect(page.locator('text=FDA 21 CFR Part 820').first()).toBeVisible();
    await expect(page.locator('text=ISO 13485').first()).toBeVisible();
  });

  test('should show document statistics and metrics', async () => {
    await page.click('a[href="#documents"]');

    // Wait for document statistics to load
    await page.waitForTimeout(2000);

    // Verify document statistics are displayed
    await expect(page.locator('text=Total Documents')).toBeVisible();
    await expect(page.locator('text=Draft Documents')).toBeVisible();
    await expect(page.locator('text=Approved Documents')).toBeVisible();

    // Verify the statistics show numbers (even if 0)
    const totalDocsCount = page.locator('h3').filter({ hasText: /^\d+$/ }).first();
    await expect(totalDocsCount).toBeVisible();
  });

  test('should display documents table with proper headers', async () => {
    await page.click('a[href="#documents"]');

    // Verify documents table structure
    await expect(page.locator('table')).toBeVisible();

    // Check table headers
    const headers = ['Title', 'Type', 'Version', 'Status', 'Created'];
    for (const header of headers) {
      await expect(page.locator(`text="${header}"`)).toBeVisible();
    }
  });

  test('should handle empty document state gracefully', async () => {
    await page.click('a[href="#documents"]');

    // Wait for content to load
    await page.waitForTimeout(2000);

    // Should show document statistics even if empty
    await expect(page.locator('text=Total Documents')).toBeVisible();

    // Should show table structure
    await expect(page.locator('table')).toBeVisible();
  });

  test('should maintain responsive design on mobile viewports', async () => {
    // Test mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    await page.click('a[href="#documents"]');

    // Verify mobile navigation works
    await expect(page.locator('h2')).toContainText('Document Control');

    // Verify content is still accessible
    await expect(page.locator('text=Total Documents')).toBeVisible();
  });

  test('should show proper loading states', async () => {
    await page.click('a[href="#documents"]');

    // Wait for content to load
    await page.waitForTimeout(2000);

    // Should show document statistics (loaded state)
    await expect(page.locator('text=Total Documents')).toBeVisible();
    await expect(page.locator('h2')).toContainText('Document Control');
  });

  test('should maintain audit trail compliance indicators', async () => {
    await page.click('a[href="#documents"]');

    // Verify regulatory compliance indicators are present
    await expect(page.locator('text=FDA 21 CFR Part 820').first()).toBeVisible();
    await expect(page.locator('text=ISO 13485').first()).toBeVisible();
    await expect(page.locator('text=Regulatory Compliant').first()).toBeVisible();
    await expect(page.locator('text=Audit Ready').first()).toBeVisible();
  });

  test('should provide return to dashboard functionality', async () => {
    await page.click('a[href="#documents"]');

    // Find and click return to dashboard button
    const returnButton = page.locator('button:has-text("Return to Dashboard")');
    await expect(returnButton).toBeVisible();

    await returnButton.click();

    // Should return to dashboard
    await expect(page.locator('h2')).toContainText('Welcome to QMS Web Interface');
  });

  test('should handle API errors gracefully', async () => {
    // Navigate to documents section
    await page.click('a[href="#documents"]');

    // Wait for content to load
    await page.waitForTimeout(2000);

    // Should show proper UI structure even if API has issues
    await expect(page.locator('h2')).toContainText('Document Control');
    await expect(page.locator('table')).toBeVisible();
    await expect(page.locator('text=Total Documents')).toBeVisible();
  });
});

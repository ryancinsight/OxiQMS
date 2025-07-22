// @ts-check
const { test, expect } = require('@playwright/test');

/**
 * QMS Navigation Tests
 * Medical Device Quality Management System - Navigation and Section Testing
 * Regulatory Compliance: FDA 21 CFR Part 820, ISO 13485, ISO 14971
 */

test.describe('QMS Navigation and Sections', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should navigate to Dashboard section', async ({ page }) => {
    // Click on Dashboard navigation link
    await page.click('nav a[href="#dashboard"]');
    
    // Wait for content to load
    await page.waitForTimeout(1000);
    
    // Take screenshot of dashboard section
    await page.screenshot({ path: 'test-results/screenshots/dashboard-section.png', fullPage: true });
    
    // Verify URL hash changed
    expect(page.url()).toContain('#dashboard');
  });

  test('should navigate to Documents section', async ({ page }) => {
    // Click on Documents navigation link
    await page.click('nav a[href="#documents"]');
    
    // Wait for content to load
    await page.waitForTimeout(1000);
    
    // Take screenshot of documents section
    await page.screenshot({ path: 'test-results/screenshots/documents-section.png', fullPage: true });
    
    // Verify URL hash changed
    expect(page.url()).toContain('#documents');
  });

  test('should navigate to Risks section', async ({ page }) => {
    // Click on Risks navigation link
    await page.click('nav a[href="#risks"]');
    
    // Wait for content to load
    await page.waitForTimeout(1000);
    
    // Take screenshot of risks section
    await page.screenshot({ path: 'test-results/screenshots/risks-section.png', fullPage: true });
    
    // Verify URL hash changed
    expect(page.url()).toContain('#risks');
  });

  test('should navigate to Requirements section', async ({ page }) => {
    // Click on Requirements navigation link
    await page.click('nav a[href="#requirements"]');
    
    // Wait for content to load
    await page.waitForTimeout(1000);
    
    // Take screenshot of requirements section
    await page.screenshot({ path: 'test-results/screenshots/requirements-section.png', fullPage: true });
    
    // Verify URL hash changed
    expect(page.url()).toContain('#requirements');
  });

  test('should handle navigation keyboard shortcuts', async ({ page }) => {
    // Test keyboard navigation (if implemented)
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');
    
    // Check that focus is on navigation elements
    const focusedElement = await page.evaluate(() => document.activeElement.tagName);
    expect(['A', 'BUTTON', 'INPUT']).toContain(focusedElement);
  });

  test('should maintain navigation state across page interactions', async ({ page }) => {
    // Navigate to different sections and verify state persistence
    const sections = ['dashboard', 'documents', 'risks', 'requirements'];
    
    for (const section of sections) {
      await page.click(`nav a[href="#${section}"]`);
      await page.waitForTimeout(500);
      
      // Verify the URL contains the correct hash
      expect(page.url()).toContain(`#${section}`);
      
      // Verify the navigation link is highlighted/active (if styling exists)
      const activeLink = page.locator(`nav a[href="#${section}"]`);
      await expect(activeLink).toBeVisible();
    }
  });

  test('should handle direct URL navigation with hash', async ({ page }) => {
    // Test direct navigation to specific sections
    await page.goto('/#documents');
    await page.waitForLoadState('networkidle');
    
    // Verify we're on the documents section
    expect(page.url()).toContain('#documents');
    
    // Take screenshot to verify correct section loaded
    await page.screenshot({ path: 'test-results/screenshots/direct-navigation-documents.png', fullPage: true });
  });
});

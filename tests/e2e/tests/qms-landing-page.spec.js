// @ts-check
const { test, expect } = require('@playwright/test');

/**
 * QMS Landing Page Tests
 * Medical Device Quality Management System - Visual and Functional Validation
 * Regulatory Compliance: FDA 21 CFR Part 820, ISO 13485, ISO 14971
 */

test.describe('QMS Landing Page', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the QMS homepage
    await page.goto('/');
    
    // Wait for the page to fully load
    await page.waitForLoadState('networkidle');
  });

  test('should load the main page with proper title and compliance indicators', async ({ page }) => {
    // Check page title
    await expect(page).toHaveTitle(/QMS.*Medical Device Quality Management System/);
    
    // Check main header
    await expect(page.locator('h1')).toContainText('QMS - Medical Device Quality Management');
    
    // Check compliance indicators in header
    await expect(page.locator('.compliance-indicator')).toContainText('FDA 21 CFR Part 820 Compliant');
    
    // Check server status indicator
    await expect(page.locator('#server-status')).toBeVisible();
    
    // Take screenshot for visual validation
    await page.screenshot({ path: 'test-results/screenshots/landing-page-full.png', fullPage: true });
  });

  test('should display all required compliance badges', async ({ page }) => {
    // Check for all compliance badges
    const badges = [
      'FDA 21 CFR Part 820',
      'ISO 13485',
      'ISO 14971',
      '21 CFR Part 11'
    ];
    
    for (const badge of badges) {
      await expect(page.locator('.badge')).toContainText(badge);
    }
    
    // Take screenshot of compliance badges
    await page.screenshot({ 
      path: 'test-results/screenshots/compliance-badges.png',
      clip: { x: 0, y: 200, width: 1200, height: 300 }
    });
  });

  test('should have functional navigation menu', async ({ page }) => {
    // Check navigation links exist and are visible
    const navLinks = ['Dashboard', 'Documents', 'Risks', 'Requirements'];
    
    for (const link of navLinks) {
      const navLink = page.locator(`nav a:has-text("${link}")`);
      await expect(navLink).toBeVisible();
      await expect(navLink).toHaveAttribute('href', `#${link.toLowerCase()}`);
    }
  });

  test('should display system information correctly', async ({ page }) => {
    // Check welcome header
    await expect(page.locator('.welcome-header h2')).toContainText('Welcome to QMS Web Interface');
    await expect(page.locator('.welcome-header p')).toContainText('Medical Device Quality Management System');
    
    // Check system info displays current time and version
    const systemInfo = page.locator('.system-info');
    await expect(systemInfo).toBeVisible();
    await expect(systemInfo).toContainText('Version 1.0.0');
    await expect(systemInfo).toContainText('Build: Production');
  });

  test('should have responsive design for mobile devices', async ({ page }) => {
    // Test mobile viewport
    await page.setViewportSize({ width: 375, height: 667 }); // iPhone SE size
    
    // Check that main elements are still visible and properly arranged
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('.compliance-badges')).toBeVisible();
    await expect(page.locator('nav')).toBeVisible();
    
    // Take mobile screenshot
    await page.screenshot({ path: 'test-results/screenshots/mobile-view.png', fullPage: true });
    
    // Test tablet viewport
    await page.setViewportSize({ width: 768, height: 1024 }); // iPad size
    await page.screenshot({ path: 'test-results/screenshots/tablet-view.png', fullPage: true });
  });

  test('should load CSS styles correctly', async ({ page }) => {
    // Check that styles are applied by verifying computed styles
    const header = page.locator('.header');
    await expect(header).toBeVisible();
    
    // Check that the page doesn't look unstyled (basic style check)
    const bodyBgColor = await page.evaluate(() => {
      return window.getComputedStyle(document.body).backgroundColor;
    });
    
    // Should not be the default white background if styles are loaded
    expect(bodyBgColor).not.toBe('rgba(0, 0, 0, 0)');
  });

  test('should handle JavaScript functionality', async ({ page }) => {
    // Check that JavaScript is working by verifying dynamic content
    await page.waitForFunction(() => {
      const timeElement = document.getElementById('current-time');
      return timeElement && timeElement.textContent && timeElement.textContent.length > 0;
    });
    
    // Verify current time is displayed and updating
    const currentTime = await page.locator('#current-time').textContent();
    expect(currentTime).toBeTruthy();
    expect(currentTime.length).toBeGreaterThan(0);
  });
});

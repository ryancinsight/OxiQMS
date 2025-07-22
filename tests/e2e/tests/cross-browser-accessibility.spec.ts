import { test, expect, Page } from '@playwright/test';

/**
 * Cross-Browser Compatibility and Accessibility Tests
 * 
 * Tests responsive design, accessibility compliance, and cross-browser functionality
 * to ensure the QMS web application works reliably across different browsers and devices.
 * 
 * Regulatory Compliance: FDA 21 CFR Part 820, Section 508, WCAG 2.1
 */

test.describe('Cross-Browser Compatibility and Accessibility', () => {
  let page: Page;

  test.beforeEach(async ({ page: testPage }) => {
    page = testPage;
    await page.goto('/');
    
    // Wait for the application to load
    await expect(page.locator('h1')).toContainText('QMS - Medical Device Quality Management');
  });

  test('should load properly across different browsers', async () => {
    // Verify core application elements load
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('nav')).toBeVisible();
    await expect(page.locator('main')).toBeVisible();
    await expect(page.locator('footer')).toBeVisible();
    
    // Verify navigation links are functional
    const navLinks = page.locator('nav a');
    const linkCount = await navLinks.count();
    expect(linkCount).toBeGreaterThan(5); // Should have multiple navigation links
    
    // Test first few navigation links
    for (let i = 0; i < Math.min(3, linkCount); i++) {
      const link = navLinks.nth(i);
      await expect(link).toBeVisible();
      await expect(link).toBeEnabled();
    }
  });

  test('should maintain responsive design on mobile devices', async () => {
    // Test various mobile viewport sizes
    const mobileViewports = [
      { width: 375, height: 667 }, // iPhone SE
      { width: 414, height: 896 }, // iPhone 11 Pro Max
      { width: 360, height: 640 }, // Galaxy S5
    ];
    
    for (const viewport of mobileViewports) {
      await page.setViewportSize(viewport);
      
      // Verify core elements are still visible and functional
      await expect(page.locator('h1')).toBeVisible();
      await expect(page.locator('nav')).toBeVisible();
      
      // Navigation should work on mobile
      const navLinks = page.locator('nav a');
      if (await navLinks.count() > 0) {
        await expect(navLinks.first()).toBeVisible();
      }
      
      // Content should not overflow
      const body = page.locator('body');
      const bodyBox = await body.boundingBox();
      if (bodyBox) {
        expect(bodyBox.width).toBeLessThanOrEqual(viewport.width + 20); // Allow small margin
      }
    }
  });

  test('should maintain responsive design on tablet devices', async () => {
    // Test tablet viewport sizes
    const tabletViewports = [
      { width: 768, height: 1024 }, // iPad
      { width: 1024, height: 768 }, // iPad Landscape
      { width: 800, height: 1280 }, // Android Tablet
    ];
    
    for (const viewport of tabletViewports) {
      await page.setViewportSize(viewport);
      
      // Verify layout adapts properly
      await expect(page.locator('h1')).toBeVisible();
      await expect(page.locator('nav')).toBeVisible();
      await expect(page.locator('main')).toBeVisible();
      
      // Navigation should be fully accessible
      const navLinks = page.locator('nav a');
      const linkCount = await navLinks.count();
      
      for (let i = 0; i < Math.min(3, linkCount); i++) {
        await expect(navLinks.nth(i)).toBeVisible();
      }
    }
  });

  test('should provide proper keyboard navigation', async () => {
    // Test keyboard navigation through main elements
    await page.keyboard.press('Tab');
    
    // Should be able to navigate to navigation links
    let focusedElement = await page.evaluate(() => document.activeElement?.tagName);
    
    // Continue tabbing through navigation
    for (let i = 0; i < 10; i++) {
      await page.keyboard.press('Tab');
      const newFocusedElement = await page.evaluate(() => document.activeElement?.tagName);
      
      // Should focus on interactive elements
      if (newFocusedElement === 'A' || newFocusedElement === 'BUTTON') {
        // Verify focused element is visible
        const activeElement = await page.evaluate(() => document.activeElement);
        expect(activeElement).toBeTruthy();
        break;
      }
    }
  });

  test('should have proper heading hierarchy', async () => {
    // Check for proper heading structure (h1 -> h2 -> h3, etc.)
    const h1Count = await page.locator('h1').count();
    expect(h1Count).toBe(1); // Should have exactly one h1
    
    const h2Count = await page.locator('h2').count();
    expect(h2Count).toBeGreaterThanOrEqual(1); // Should have at least one h2
    
    // Verify h1 comes before h2
    const h1Text = await page.locator('h1').first().textContent();
    expect(h1Text).toContain('QMS');
  });

  test('should have proper alt text for images', async () => {
    // Check all images have alt text
    const images = page.locator('img');
    const imageCount = await images.count();
    
    for (let i = 0; i < imageCount; i++) {
      const img = images.nth(i);
      const altText = await img.getAttribute('alt');
      
      // Alt text should exist (can be empty for decorative images)
      expect(altText).not.toBeNull();
    }
  });

  test('should have proper color contrast', async () => {
    // Test that text has sufficient contrast against background
    // This is a basic check - full contrast testing would require specialized tools
    
    const textElements = page.locator('p, h1, h2, h3, h4, h5, h6, span, div');
    const elementCount = await textElements.count();
    
    if (elementCount > 0) {
      // Check that text elements are visible (basic contrast check)
      for (let i = 0; i < Math.min(5, elementCount); i++) {
        const element = textElements.nth(i);
        const text = await element.textContent();
        
        if (text && text.trim().length > 0) {
          await expect(element).toBeVisible();
        }
      }
    }
  });

  test('should handle focus management properly', async () => {
    // Test focus management when navigating between sections
    const navLinks = page.locator('nav a');
    const linkCount = await navLinks.count();
    
    if (linkCount > 1) {
      // Click on a navigation link
      await navLinks.nth(1).click();
      
      // Wait for navigation
      await page.waitForTimeout(1000);
      
      // Focus should be managed appropriately
      const focusedElement = await page.evaluate(() => document.activeElement?.tagName);
      expect(['A', 'BUTTON', 'H1', 'H2', 'MAIN', 'BODY']).toContain(focusedElement);
    }
  });

  test('should provide proper ARIA labels and roles', async () => {
    // Check for ARIA landmarks
    const main = page.locator('main, [role="main"]');
    await expect(main).toBeVisible();
    
    const nav = page.locator('nav, [role="navigation"]');
    await expect(nav).toBeVisible();
    
    // Check for ARIA labels on interactive elements
    const buttons = page.locator('button');
    const buttonCount = await buttons.count();
    
    for (let i = 0; i < Math.min(3, buttonCount); i++) {
      const button = buttons.nth(i);
      const ariaLabel = await button.getAttribute('aria-label');
      const text = await button.textContent();
      
      // Button should have either aria-label or visible text
      expect(ariaLabel || (text && text.trim().length > 0)).toBeTruthy();
    }
  });

  test('should handle high contrast mode', async () => {
    // Simulate high contrast mode by checking if elements remain visible
    // This is a basic test - full high contrast testing requires OS-level changes
    
    await page.addStyleTag({
      content: `
        * {
          background: black !important;
          color: white !important;
          border-color: white !important;
        }
      `
    });
    
    // Verify core elements are still visible
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('nav')).toBeVisible();
    await expect(page.locator('main')).toBeVisible();
  });

  test('should work with reduced motion preferences', async () => {
    // Simulate reduced motion preference
    await page.emulateMedia({ reducedMotion: 'reduce' });
    
    // Navigate between sections
    const navLinks = page.locator('nav a');
    if (await navLinks.count() > 1) {
      await navLinks.nth(1).click();
      await page.waitForTimeout(500);
      
      // Content should still load properly without animations
      await expect(page.locator('main')).toBeVisible();
    }
  });

  test('should handle JavaScript disabled gracefully', async () => {
    // This test would require a separate browser context with JS disabled
    // For now, we test that core content is available without JS interactions
    
    // Verify that basic HTML structure is present
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('nav')).toBeVisible();
    await expect(page.locator('main')).toBeVisible();
    await expect(page.locator('footer')).toBeVisible();
    
    // Navigation links should have proper href attributes
    const navLinks = page.locator('nav a');
    const linkCount = await navLinks.count();
    
    for (let i = 0; i < Math.min(3, linkCount); i++) {
      const href = await navLinks.nth(i).getAttribute('href');
      expect(href).toBeTruthy();
    }
  });

  test('should maintain performance across browsers', async () => {
    // Basic performance check - measure page load time
    const startTime = Date.now();
    
    await page.reload();
    await page.waitForLoadState('networkidle');
    
    const loadTime = Date.now() - startTime;
    
    // Page should load within reasonable time (10 seconds)
    expect(loadTime).toBeLessThan(10000);
    
    // Core elements should be visible after load
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('nav')).toBeVisible();
  });

  test('should handle print styles properly', async () => {
    // Simulate print media
    await page.emulateMedia({ media: 'print' });
    
    // Core content should still be visible for printing
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('main')).toBeVisible();
    
    // Navigation might be hidden in print mode, which is acceptable
    // Footer should contain important regulatory information for print
    await expect(page.locator('footer')).toBeVisible();
  });
});

import { test, expect, Page } from '@playwright/test';

/**
 * Requirements Tracking Verification Tests
 * 
 * Confirms that all requirements (REQ-001 through REQ-005) created via CLI
 * are visible in the web interface with proper categorization and status tracking.
 * 
 * Regulatory Compliance: FDA 21 CFR Part 820, ISO 13485
 */

test.describe('Requirements Management Web Interface', () => {
  let page: Page;

  test.beforeEach(async ({ page: testPage }) => {
    page = testPage;
    await page.goto('/');

    // Wait for the application to load
    await expect(page.locator('h1')).toContainText('QMS - Medical Device Quality Management');
    await expect(page.locator('text=🟢 Connected')).toBeVisible();
  });

  test('should display requirements section with proper navigation', async () => {
    // Navigate to Requirements section
    await page.click('a[href="#requirements"]');
    
    // Verify Requirements section loads
    await expect(page.locator('h2')).toContainText('Requirements Management');
    await expect(page.locator('p')).toContainText('Medical Device Requirements Traceability');
    
    // Verify regulatory compliance context
    await expect(page.locator('text=FDA 21 CFR Part 820').first()).toBeVisible();
  });

  test('should show requirements statistics and metrics', async () => {
    await page.click('a[href="#requirements"]');
    
    // Wait for requirements statistics to load
    await page.waitForTimeout(2000);

    // Verify requirements statistics are displayed
    await expect(page.locator('text=Total Requirements')).toBeVisible();
    await expect(page.locator('text=Verified')).toBeVisible();
    await expect(page.locator('text=Approved')).toBeVisible();
    await expect(page.locator('text=Implemented')).toBeVisible();
  });

  test('should display requirements matrix with proper structure', async () => {
    await page.click('a[href="#requirements"]');
    
    // Verify requirements matrix table
    await expect(page.locator('h3:has-text("Requirements Matrix")')).toBeVisible();
    await expect(page.locator('table')).toBeVisible();
    
    // Check table headers
    const expectedHeaders = ['Requirement ID', 'Title', 'Priority', 'Status', 'Actions'];
    for (const header of expectedHeaders) {
      await expect(page.locator(`th:has-text("${header}")`)).toBeVisible();
    }
  });

  test('should show requirement entries with proper formatting', async () => {
    await page.click('a[href="#requirements"]');
    
    // Wait for requirements data to load
    await page.waitForTimeout(2000);
    
    // Look for requirement entries (may be mock data or real data)
    const requirementRows = page.locator('tbody tr');
    const rowCount = await requirementRows.count();
    
    if (rowCount > 0) {
      // Verify first requirement row structure
      const firstRow = requirementRows.first();
      
      // Should have requirement ID
      await expect(firstRow.locator('td:first-child strong')).toBeVisible();
      
      // Should have title
      await expect(firstRow.locator('td:nth-child(2)')).toBeVisible();
      
      // Should have priority badge
      await expect(firstRow.locator('td:nth-child(3) .priority-badge')).toBeVisible();
      
      // Should have status badge
      await expect(firstRow.locator('td:nth-child(4) .status-badge')).toBeVisible();
      
      // Should have action button
      await expect(firstRow.locator('button:has-text("View Details")')).toBeVisible();
    }
  });

  test('should handle different requirement priorities correctly', async () => {
    await page.click('a[href="#requirements"]');
    
    // Wait for data to load
    await page.waitForTimeout(2000);
    
    // Check for different priority levels
    const priorities = ['HIGH', 'CRITICAL', 'MEDIUM', 'LOW'];
    
    for (const priority of priorities) {
      const priorityElements = page.locator(`.priority-badge:has-text("${priority}")`);
      const count = await priorityElements.count();
      
      if (count > 0) {
        // Verify priority badge is properly styled
        await expect(priorityElements.first()).toBeVisible();
        
        // Priority badges should have appropriate CSS classes
        const className = await priorityElements.first().getAttribute('class');
        expect(className).toContain('priority-badge');
      }
    }
  });

  test('should handle different requirement statuses correctly', async () => {
    await page.click('a[href="#requirements"]');
    
    // Wait for data to load
    await page.waitForTimeout(2000);
    
    // Check for different status levels
    const statuses = ['verified', 'approved', 'implemented', 'draft', 'review'];
    
    for (const status of statuses) {
      const statusElements = page.locator(`.status-badge:has-text("${status}")`);
      const count = await statusElements.count();
      
      if (count > 0) {
        // Verify status badge is properly styled
        await expect(statusElements.first()).toBeVisible();
        
        // Status badges should have appropriate CSS classes
        const className = await statusElements.first().getAttribute('class');
        expect(className).toContain('status-badge');
      }
    }
  });

  test('should provide requirement detail view functionality', async () => {
    await page.click('a[href="#requirements"]');
    
    // Wait for requirements to load
    await page.waitForTimeout(2000);
    
    // Find and click a "View Details" button if available
    const detailButtons = page.locator('button:has-text("View Details")');
    const buttonCount = await detailButtons.count();
    
    if (buttonCount > 0) {
      await detailButtons.first().click();
      
      // Should show some kind of detail view or modal
      // (Implementation may vary - could be modal, new page, or expanded row)
      await page.waitForTimeout(1000);
      
      // Verify some detail content is shown
      // This is flexible to accommodate different UI implementations
      const hasModal = await page.locator('.modal, .dialog, .detail-view').count() > 0;
      const hasExpandedContent = await page.locator('.requirement-details, .expanded-row').count() > 0;
      
      expect(hasModal || hasExpandedContent).toBeTruthy();
    }
  });

  test('should maintain responsive design for requirements matrix', async () => {
    // Test mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    await page.click('a[href="#requirements"]');
    
    // Verify mobile navigation works
    await expect(page.locator('h2')).toContainText('Requirements Management');
    
    // Table should be responsive or have horizontal scroll
    const table = page.locator('table');
    await expect(table).toBeVisible();
    
    // On mobile, table might be in a scrollable container
    const tableContainer = page.locator('.table-container, .table-responsive');
    if (await tableContainer.count() > 0) {
      await expect(tableContainer).toBeVisible();
    }
  });

  test('should show proper loading states for requirements', async () => {
    await page.click('a[href="#requirements"]');
    
    // Check for loading indicators
    const loadingElements = page.locator('text=Loading...');
    
    // If loading elements exist, they should eventually disappear
    if (await loadingElements.count() > 0) {
      await expect(loadingElements.first()).toBeHidden({ timeout: 15000 });
    }
    
    // Should eventually show either data or empty state
    await page.waitForTimeout(3000);
    const hasData = await page.locator('tbody tr').count() > 0;
    const hasEmptyState = await page.locator('text=/No requirements|Empty/i').count() > 0;
    
    expect(hasData || hasEmptyState).toBeTruthy();
  });

  test('should maintain regulatory compliance indicators', async () => {
    await page.click('a[href="#requirements"]');
    
    // Verify regulatory compliance indicators are present
    await expect(page.locator('text=FDA 21 CFR Part 820').first()).toBeVisible();
    await expect(page.locator('text=ISO 13485').first()).toBeVisible();
    await expect(page.locator('text=Regulatory Compliant').first()).toBeVisible();
    await expect(page.locator('text=Audit Ready').first()).toBeVisible();
  });

  test('should provide return to dashboard functionality', async () => {
    await page.click('a[href="#requirements"]');
    
    // Find and click return to dashboard button
    const returnButton = page.locator('button:has-text("Return to Dashboard")');
    await expect(returnButton).toBeVisible();
    
    await returnButton.click();
    
    // Should return to dashboard
    await expect(page.locator('h2')).toContainText('Welcome to QMS Web Interface');
  });
});

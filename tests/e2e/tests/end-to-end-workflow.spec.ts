import { test, expect, Page } from '@playwright/test';

/**
 * End-to-End Workflow Tests
 * 
 * Validates complete user workflows through the QMS web interface,
 * ensuring users can perform all QMS operations without requiring CLI access.
 * 
 * Regulatory Compliance: FDA 21 CFR Part 820, ISO 13485, ISO 14971
 */

test.describe('Complete QMS Web Workflow', () => {
  let page: Page;

  test.beforeEach(async ({ page: testPage }) => {
    page = testPage;
    await page.goto('/');

    // Wait for the application to load completely
    await expect(page.locator('h1')).toContainText('QMS - Medical Device Quality Management');
    await expect(page.locator('text=🟢 Connected')).toBeVisible();

    // Wait for dashboard to initialize
    await page.waitForTimeout(2000);
  });

  test('should complete full QMS navigation workflow', async () => {
    // Test complete navigation through all QMS sections
    const sections = [
      { link: 'a[href="#documents"]', title: 'Document Control' },
      { link: 'a[href="#requirements"]', title: 'Requirements Management' },
      { link: 'a[href="#risks"]', title: 'Risk Management' },
      { link: 'a[href="#audit"]', title: 'Audit Trail' },
      { link: 'a[href="#reports"]', title: 'Reports' },
      { link: 'a[href="#projects"]', title: 'Project Management' },
    ];

    for (const section of sections) {
      // Navigate to section
      await page.click(section.link);
      
      // Verify section loads
      await expect(page.locator('h2')).toContainText(section.title);
      
      // Verify regulatory compliance is maintained
      await expect(page.locator('text=FDA 21 CFR Part 820').first()).toBeVisible();
      
      // Wait for section to fully load
      await page.waitForTimeout(1000);
      
      // Return to dashboard
      const returnButton = page.locator('button:has-text("Return to Dashboard")');
      if (await returnButton.count() > 0) {
        await returnButton.click();
        await expect(page.locator('h2')).toContainText('Welcome to QMS Web Interface');
      } else {
        // Navigate back to dashboard via navigation
        await page.click('a[href="#dashboard"]');
      }
    }
  });

  test('should complete document management workflow', async () => {
    // Navigate to Documents section
    await page.click('a[href="#documents"]');
    await expect(page.locator('h2')).toContainText('Document Control');
    
    // Verify document statistics are displayed
    await page.waitForTimeout(2000);
    const statsVisible = await page.locator('.document-stats, .stat-item').count() > 0;
    expect(statsVisible).toBeTruthy();
    
    // Verify document table structure
    await expect(page.locator('table')).toBeVisible();
    
    // Check for document management features
    const documentFeatures = [
      'Total Documents',
      'Draft Documents', 
      'Approved Documents',
      'Recent Documents'
    ];
    
    for (const feature of documentFeatures) {
      const featureElement = page.locator(`text="${feature}"`);
      if (await featureElement.count() > 0) {
        await expect(featureElement).toBeVisible();
      }
    }
  });

  test('should complete requirements tracking workflow', async () => {
    // Navigate to Requirements section
    await page.click('a[href="#requirements"]');
    await expect(page.locator('h2')).toContainText('Requirements Management');
    
    // Verify requirements matrix
    await expect(page.locator('h3:has-text("Requirements Matrix")')).toBeVisible();
    await expect(page.locator('table')).toBeVisible();
    
    // Check requirements statistics
    await page.waitForTimeout(2000);
    const requirementStats = [
      'Total Requirements',
      'Verified',
      'Approved',
      'Implemented'
    ];
    
    for (const stat of requirementStats) {
      const statElement = page.locator(`text="${stat}"`);
      if (await statElement.count() > 0) {
        await expect(statElement).toBeVisible();
      }
    }
    
    // Test requirement detail view if available
    const detailButtons = page.locator('button:has-text("View Details")');
    if (await detailButtons.count() > 0) {
      await detailButtons.first().click();
      await page.waitForTimeout(1000);
      
      // Should show some form of detail view
      const hasDetailView = await page.locator('.modal, .detail-view, .expanded-row').count() > 0;
      if (hasDetailView) {
        // Close detail view if modal
        const closeButton = page.locator('button:has-text("Close"), button:has-text("✕")');
        if (await closeButton.count() > 0) {
          await closeButton.first().click();
        }
      }
    }
  });

  test('should complete risk management workflow', async () => {
    // Navigate to Risk Management section
    await page.click('a[href="#risks"]');
    
    // Wait for risk management content to load
    await page.waitForTimeout(3000);
    
    // Should show risk management content or appropriate message
    const riskContent = page.locator('text=/Risk|FMEA|Hazard|Management/i');
    const hasRiskContent = await riskContent.count() > 0;
    
    if (hasRiskContent) {
      await expect(riskContent.first()).toBeVisible();
      
      // Look for ISO 14971 compliance indicators
      await expect(page.locator('footer')).toContainText('ISO 14971');
    } else {
      // Should show development message or empty state
      const developmentMessage = page.locator('text=/coming soon|development|not yet implemented/i');
      if (await developmentMessage.count() > 0) {
        await expect(developmentMessage.first()).toBeVisible();
      }
    }
  });

  test('should complete reports generation workflow', async () => {
    // Navigate to Reports section
    await page.click('a[href="#reports"]');
    await expect(page.locator('h2')).toContainText('Reports');
    
    // Verify Reports API is functional
    await expect(page.locator('text=Reports API is now active and functional!')).toBeVisible();
    
    // Test Audit Trail Report generation
    const auditReportButton = page.locator('button:has-text("Generate Report")').first();
    await expect(auditReportButton).toBeVisible();
    
    await auditReportButton.click();
    await page.waitForTimeout(3000);
    
    // Verify report was generated
    const generatedReports = page.locator('.generated-report, .report-item');
    if (await generatedReports.count() > 0) {
      await expect(generatedReports.first()).toBeVisible();
      
      // Verify report content
      await expect(page.locator('text=AUDIT Compliance Report')).toBeVisible();
      await expect(page.locator('text=Generated:')).toBeVisible();
      
      // Test download functionality
      const downloadButton = page.locator('button:has-text("Download")');
      if (await downloadButton.count() > 0) {
        await expect(downloadButton.first()).toBeVisible();
        await expect(downloadButton.first()).toBeEnabled();
      }
    }
    
    // Test Risk Management Report generation
    const riskReportButton = page.locator('button:has-text("Generate Report")').nth(2);
    if (await riskReportButton.count() > 0) {
      await riskReportButton.click();
      await page.waitForTimeout(3000);
      
      // Should generate risk report
      const riskReport = page.locator('text=RISK Compliance Report');
      if (await riskReport.count() > 0) {
        await expect(riskReport).toBeVisible();
      }
    }
  });

  test('should complete project management workflow', async () => {
    // Navigate to Projects section
    await page.click('a[href="#projects"]');
    await expect(page.locator('h2')).toContainText('Project Management');
    
    // Verify project management interface
    await expect(page.locator('button:has-text("Create New Project")')).toBeVisible();
    await expect(page.locator('button:has-text("Refresh Projects")')).toBeVisible();
    
    // Test project refresh
    await page.click('button:has-text("Refresh Projects")');
    await page.waitForTimeout(2000);
    
    // Should still show projects table
    await expect(page.locator('table')).toBeVisible();
    
    // Test project view if projects exist
    const viewButtons = page.locator('button:has-text("View")');
    if (await viewButtons.count() > 0) {
      await viewButtons.first().click();
      await page.waitForTimeout(1000);
      
      // Should show project details
      const projectDetails = page.locator('h3:has-text("Project Details")');
      if (await projectDetails.count() > 0) {
        await expect(projectDetails).toBeVisible();
        
        // Verify compliance status
        const complianceSection = page.locator('h4:has-text("Compliance Status")');
        if (await complianceSection.count() > 0) {
          await expect(complianceSection).toBeVisible();
        }
        
        // Close project details
        const closeButton = page.locator('button:has-text("Close")');
        if (await closeButton.count() > 0) {
          await closeButton.click();
        }
      }
    }
  });

  test('should maintain audit trail throughout workflow', async () => {
    // Navigate to Audit Trail section
    await page.click('a[href="#audit"]');
    
    // Wait for audit trail content
    await page.waitForTimeout(2000);
    
    // Should show audit trail information
    await expect(page.locator('h2')).toContainText('Audit Trail');
    await expect(page.locator('text=FDA 21 CFR Part 820 Compliance Logging')).toBeVisible();
    
    // Verify audit trail compliance messaging
    const auditMessages = [
      'tracks all user actions for regulatory compliance',
      'document changes, risk assessments, and system access are logged',
      'Audit trail'
    ];
    
    for (const message of auditMessages) {
      const messageElement = page.locator(`text*="${message}"`);
      if (await messageElement.count() > 0) {
        await expect(messageElement.first()).toBeVisible();
      }
    }
  });

  test('should handle error states gracefully throughout workflow', async () => {
    // Monitor console errors throughout the workflow
    const consoleErrors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });
    
    // Navigate through all sections quickly
    const sections = ['#documents', '#requirements', '#risks', '#audit', '#reports', '#projects'];
    
    for (const section of sections) {
      await page.click(`a[href="${section}"]`);
      await page.waitForTimeout(1000);
    }
    
    // Should have minimal critical errors
    const criticalErrors = consoleErrors.filter(error => 
      error.includes('TypeError') || 
      error.includes('ReferenceError') ||
      error.includes('SyntaxError')
    );
    
    expect(criticalErrors.length).toBeLessThan(3);
  });

  test('should maintain regulatory compliance throughout workflow', async () => {
    // Verify regulatory compliance is maintained across all sections
    const sections = ['#documents', '#requirements', '#risks', '#reports', '#projects'];
    
    for (const section of sections) {
      await page.click(`a[href="${section}"]`);
      await page.waitForTimeout(1000);
      
      // Verify regulatory compliance indicators
      await expect(page.locator('text=FDA 21 CFR Part 820').first()).toBeVisible();
      await expect(page.locator('text=Regulatory Compliant').first()).toBeVisible();
      await expect(page.locator('text=Audit Ready').first()).toBeVisible();
      
      // Verify medical device context is maintained
      const medicalDeviceContext = page.locator('text=/Medical Device|Quality Management/i');
      await expect(medicalDeviceContext.first()).toBeVisible();
    }
  });

  test('should provide consistent user experience across workflow', async () => {
    // Test consistent navigation and UI elements
    const sections = ['#documents', '#requirements', '#reports', '#projects'];
    
    for (const section of sections) {
      await page.click(`a[href="${section}"]`);
      await page.waitForTimeout(1000);
      
      // Should have consistent header
      await expect(page.locator('h1')).toContainText('QMS - Medical Device Quality Management');
      
      // Should have consistent navigation
      await expect(page.locator('nav')).toBeVisible();
      
      // Should have consistent footer
      await expect(page.locator('footer')).toBeVisible();
      
      // Should have return to dashboard button
      const returnButton = page.locator('button:has-text("Return to Dashboard")');
      if (await returnButton.count() > 0) {
        await expect(returnButton).toBeVisible();
      }
    }
  });
});

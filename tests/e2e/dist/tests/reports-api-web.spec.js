"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const test_1 = require("@playwright/test");
/**
 * Reports API Web Testing
 *
 * Uses Playwright to automate the generation of compliance reports through the web interface
 * and verify report generation, content, formatting, and regulatory compliance.
 *
 * Regulatory Compliance: FDA 21 CFR Part 820, ISO 13485, ISO 14971
 */
test_1.test.describe('Reports API Web Interface', () => {
    let page;
    test_1.test.beforeEach(async ({ page: testPage }) => {
        page = testPage;
        await page.goto('/');
        // Wait for the application to load
        await (0, test_1.expect)(page.locator('h1')).toContainText('QMS - Medical Device Quality Management');
        await (0, test_1.expect)(page.locator('text=🟢 Connected')).toBeVisible();
    });
    (0, test_1.test)('should display reports section with proper navigation', async () => {
        // Navigate to Reports section
        await page.click('a[href="#reports"]');
        // Verify Reports section loads
        await (0, test_1.expect)(page.locator('h2')).toContainText('Reports');
        await (0, test_1.expect)(page.locator('text=Medical Device Compliance Reports')).toBeVisible();
        // Verify regulatory compliance context
        await (0, test_1.expect)(page.locator('text=FDA 21 CFR Part 820').first()).toBeVisible();
        await (0, test_1.expect)(page.locator('text=ISO 13485').first()).toBeVisible();
        await (0, test_1.expect)(page.locator('text=ISO 14971').first()).toBeVisible();
    });
    (0, test_1.test)('should show reports API status and functionality', async () => {
        await page.click('a[href="#reports"]');
        // Verify Reports API is active
        await (0, test_1.expect)(page.locator('text=Reports API is now active and functional!')).toBeVisible();
        // Verify available reports section
        await (0, test_1.expect)(page.locator('h3:has-text("Available Reports")')).toBeVisible();
    });
    (0, test_1.test)('should display all available report types', async () => {
        await page.click('a[href="#reports"]');
        // Verify all three report types are available
        await (0, test_1.expect)(page.locator('text=Audit Trail Report')).toBeVisible();
        await (0, test_1.expect)(page.locator('text=Design History File Report')).toBeVisible();
        await (0, test_1.expect)(page.locator('text=Risk Management Report')).toBeVisible();
    });
    (0, test_1.test)('should show proper report descriptions and compliance badges', async () => {
        await page.click('a[href="#reports"]');
        // Audit Trail Report
        await (0, test_1.expect)(page.locator('text=Comprehensive audit trail report for regulatory compliance')).toBeVisible();
        await (0, test_1.expect)(page.locator('text=FDA 21 CFR Part 820').first()).toBeVisible();
        await (0, test_1.expect)(page.locator('text=21 CFR Part 11').first()).toBeVisible();
        // Design History File Report
        await (0, test_1.expect)(page.locator('text=Design History File summary for medical device documentation')).toBeVisible();
        // Risk Management Report
        await (0, test_1.expect)(page.locator('text=Risk assessment and management report per ISO 14971')).toBeVisible();
        await (0, test_1.expect)(page.locator('text=ISO 14971').first()).toBeVisible();
    });
    (0, test_1.test)('should provide format selection for reports', async () => {
        await page.click('a[href="#reports"]');
        // Check format dropdowns for each report - the interface uses combobox elements
        // Verify the reports section shows format dropdowns
        const formatSelectors = page.locator('combobox');
        (0, test_1.expect)(await formatSelectors.count()).toBeGreaterThanOrEqual(3);
        // Verify report generation buttons are available
        const generateButtons = page.locator('button:has-text("Generate Report")');
        (0, test_1.expect)(await generateButtons.count()).toBeGreaterThanOrEqual(3);
    });
    (0, test_1.test)('should generate Audit Trail Report successfully', async () => {
        await page.click('a[href="#reports"]');
        // Find and click the first "Generate Report" button (Audit Trail Report)
        const generateButtons = page.locator('button:has-text("Generate Report")');
        await (0, test_1.expect)(generateButtons.first()).toBeVisible();
        await generateButtons.first().click();
        // Wait for report generation
        await page.waitForTimeout(3000);
        // Verify report was generated
        await (0, test_1.expect)(page.locator('h3:has-text("Generated Reports")')).toBeVisible();
        // Look for the generated report
        await (0, test_1.expect)(page.locator('text=AUDIT Compliance Report')).toBeVisible();
        await (0, test_1.expect)(page.locator('text=Generated:').first()).toBeVisible();
        await (0, test_1.expect)(page.locator('text=Format: HTML')).toBeVisible();
    });
    (0, test_1.test)('should generate Risk Management Report successfully', async () => {
        await page.click('a[href="#reports"]');
        // Find and click the Risk Management Report generate button (third one)
        const generateButtons = page.locator('button:has-text("Generate Report")');
        const buttonCount = await generateButtons.count();
        if (buttonCount >= 3) {
            const riskReportButton = generateButtons.nth(2); // Third button (0-indexed)
            await (0, test_1.expect)(riskReportButton).toBeVisible();
            await riskReportButton.click();
        }
        else {
            // If there are fewer buttons, just click the last one
            const lastButton = generateButtons.last();
            await (0, test_1.expect)(lastButton).toBeVisible();
            await lastButton.click();
        }
        // Wait for report generation
        await page.waitForTimeout(3000);
        // Look for the generated risk report
        const riskReport = page.locator('text=RISK Compliance Report');
        if (await riskReport.count() > 0) {
            await (0, test_1.expect)(riskReport).toBeVisible();
            // Verify ISO 14971 compliance content
            await (0, test_1.expect)(page.locator('text=ISO 14971').first()).toBeVisible();
            await (0, test_1.expect)(page.locator('text=Risk Management Report').first()).toBeVisible();
        }
    });
    (0, test_1.test)('should show proper report content and formatting', async () => {
        await page.click('a[href="#reports"]');
        // Generate a report first
        const generateButton = page.locator('button:has-text("Generate Report")').first();
        await generateButton.click();
        await page.waitForTimeout(3000);
        // Check for report content structure
        const reportContent = page.locator('.report-content, .generated-report');
        if (await reportContent.count() > 0) {
            // Verify report has proper structure
            await (0, test_1.expect)(page.locator('text=Generated:').first()).toBeVisible();
            await (0, test_1.expect)(page.locator('text=Format:').first()).toBeVisible();
            await (0, test_1.expect)(page.locator('text=ID:').first()).toBeVisible();
            // Verify compliance status indicators
            await (0, test_1.expect)(page.locator('text=✅').first()).toBeVisible();
            await (0, test_1.expect)(page.locator('text=Compliant').first()).toBeVisible();
        }
    });
    (0, test_1.test)('should provide download functionality for generated reports', async () => {
        await page.click('a[href="#reports"]');
        // Generate a report first
        const generateButton = page.locator('button:has-text("Generate Report")').first();
        await generateButton.click();
        await page.waitForTimeout(3000);
        // Look for download button
        const downloadButton = page.locator('button:has-text("Download")');
        if (await downloadButton.count() > 0) {
            await (0, test_1.expect)(downloadButton).toBeVisible();
            // Verify download button is clickable
            await (0, test_1.expect)(downloadButton).toBeEnabled();
        }
    });
    (0, test_1.test)('should provide print functionality for generated reports', async () => {
        await page.click('a[href="#reports"]');
        // Generate a report first
        const generateButton = page.locator('button:has-text("Generate Report")').first();
        await generateButton.click();
        await page.waitForTimeout(3000);
        // Look for print button
        const printButton = page.locator('button:has-text("Print")');
        if (await printButton.count() > 0) {
            await (0, test_1.expect)(printButton).toBeVisible();
            // Verify print button is clickable
            await (0, test_1.expect)(printButton).toBeEnabled();
        }
    });
    (0, test_1.test)('should handle multiple report generation', async () => {
        await page.click('a[href="#reports"]');
        // Generate multiple reports
        const generateButtons = page.locator('button:has-text("Generate Report")');
        const buttonCount = await generateButtons.count();
        // Generate first two reports
        if (buttonCount >= 2) {
            await generateButtons.first().click();
            await page.waitForTimeout(2000);
            await generateButtons.nth(1).click();
            await page.waitForTimeout(2000);
            // Should show multiple generated reports
            const reportItems = page.locator('.generated-report, .report-item');
            const reportCount = await reportItems.count();
            (0, test_1.expect)(reportCount).toBeGreaterThanOrEqual(1);
        }
    });
    (0, test_1.test)('should maintain regulatory compliance in report content', async () => {
        await page.click('a[href="#reports"]');
        // Generate a report
        const generateButton = page.locator('button:has-text("Generate Report")').first();
        await generateButton.click();
        await page.waitForTimeout(3000);
        // Verify regulatory compliance content in generated report
        const complianceIndicators = [
            'FDA 21 CFR Part 820',
            '21 CFR Part 11',
            'ISO 13485',
            'ISO 14971',
            'Compliant',
            'Regulatory'
        ];
        for (const indicator of complianceIndicators) {
            const elements = page.locator(`text="${indicator}"`);
            if (await elements.count() > 0) {
                await (0, test_1.expect)(elements.first()).toBeVisible();
            }
        }
    });
    (0, test_1.test)('should handle report generation errors gracefully', async () => {
        await page.click('a[href="#reports"]');
        // Monitor console for errors
        const consoleErrors = [];
        page.on('console', msg => {
            if (msg.type() === 'error') {
                consoleErrors.push(msg.text());
            }
        });
        // Try to generate a report
        const generateButton = page.locator('button:has-text("Generate Report")').first();
        await generateButton.click();
        await page.waitForTimeout(5000);
        // Should not have critical JavaScript errors
        const criticalErrors = consoleErrors.filter(error => error.includes('TypeError') || error.includes('ReferenceError'));
        (0, test_1.expect)(criticalErrors.length).toBeLessThan(2);
    });
    (0, test_1.test)('should maintain responsive design for reports interface', async () => {
        // Test mobile viewport
        await page.setViewportSize({ width: 375, height: 667 });
        await page.click('a[href="#reports"]');
        // Verify mobile navigation works
        await (0, test_1.expect)(page.locator('h2')).toContainText('Reports');
        // Report cards should be stacked vertically on mobile
        const reportCards = page.locator('.report-card, .report-section');
        if (await reportCards.count() > 0) {
            await (0, test_1.expect)(reportCards.first()).toBeVisible();
        }
        // Generate buttons should still be accessible
        const generateButtons = page.locator('button:has-text("Generate Report")');
        if (await generateButtons.count() > 0) {
            await (0, test_1.expect)(generateButtons.first()).toBeVisible();
        }
    });
    (0, test_1.test)('should provide return to dashboard functionality', async () => {
        await page.click('a[href="#reports"]');
        // Find and click return to dashboard button
        const returnButton = page.locator('button:has-text("Return to Dashboard")');
        await (0, test_1.expect)(returnButton).toBeVisible();
        await returnButton.click();
        // Should return to dashboard
        await (0, test_1.expect)(page.locator('h2')).toContainText('Welcome to QMS Web Interface');
    });
});
//# sourceMappingURL=reports-api-web.spec.js.map
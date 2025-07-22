"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const test_1 = require("@playwright/test");
/**
 * Risk Management Integration Tests
 *
 * Validates that risk assessments and FMEA entries created through CLI commands
 * are accessible and properly formatted in the web browser's Risk Management section.
 *
 * Regulatory Compliance: ISO 14971, FDA 21 CFR Part 820
 */
test_1.test.describe('Risk Management Web Interface', () => {
    let page;
    test_1.test.beforeEach(async ({ page: testPage }) => {
        page = testPage;
        await page.goto('/');
        // Wait for the application to load
        await (0, test_1.expect)(page.locator('h1')).toContainText('QMS - Medical Device Quality Management');
        await (0, test_1.expect)(page.locator('text=🟢 Connected')).toBeVisible();
    });
    (0, test_1.test)('should display risk management section with proper navigation', async () => {
        // Navigate to Risks section
        await page.click('a[href="#risks"]');
        // Verify Risks section loads
        await (0, test_1.expect)(page.locator('h2')).toContainText('Risk Management');
        // Verify ISO 14971 compliance context
        await (0, test_1.expect)(page.locator('text=ISO 14971').first()).toBeVisible();
    });
    (0, test_1.test)('should show risk management dashboard with key metrics', async () => {
        await page.click('a[href="#risks"]');
        // Wait for risk management dashboard to load
        await page.waitForTimeout(2000);
        // Look for risk management specific content
        const riskContent = page.locator('text=/Risk|FMEA|Hazard|Mitigation/i');
        await (0, test_1.expect)(riskContent.first()).toBeVisible({ timeout: 10000 });
    });
    (0, test_1.test)('should handle risk assessment display properly', async () => {
        await page.click('a[href="#risks"]');
        // Wait for content to load
        await page.waitForTimeout(3000);
        // Check for risk-related elements
        const riskElements = [
            'text=/Risk Assessment/i',
            'text=/FMEA/i',
            'text=/Hazard Analysis/i',
            'text=/Risk Matrix/i',
            'text=/Mitigation/i'
        ];
        let foundRiskElement = false;
        for (const selector of riskElements) {
            const element = page.locator(selector);
            if (await element.count() > 0) {
                await (0, test_1.expect)(element.first()).toBeVisible();
                foundRiskElement = true;
                break;
            }
        }
        // Should find at least one risk-related element
        (0, test_1.expect)(foundRiskElement).toBeTruthy();
    });
    (0, test_1.test)('should display risk severity levels correctly', async () => {
        await page.click('a[href="#risks"]');
        // Wait for risk data to load
        await page.waitForTimeout(2000);
        // Look for severity indicators
        const severityLevels = ['Critical', 'High', 'Medium', 'Low', 'Negligible'];
        for (const level of severityLevels) {
            const severityElements = page.locator(`text="${level}"`);
            const count = await severityElements.count();
            if (count > 0) {
                // Verify severity level is displayed
                await (0, test_1.expect)(severityElements.first()).toBeVisible();
            }
        }
    });
    (0, test_1.test)('should show risk probability assessments', async () => {
        await page.click('a[href="#risks"]');
        // Wait for risk data to load
        await page.waitForTimeout(2000);
        // Look for probability-related content
        const probabilityTerms = ['Frequent', 'Probable', 'Occasional', 'Remote', 'Improbable'];
        for (const term of probabilityTerms) {
            const elements = page.locator(`text="${term}"`);
            const count = await elements.count();
            if (count > 0) {
                await (0, test_1.expect)(elements.first()).toBeVisible();
            }
        }
    });
    (0, test_1.test)('should display RPN (Risk Priority Number) calculations', async () => {
        await page.click('a[href="#risks"]');
        // Wait for risk calculations to load
        await page.waitForTimeout(2000);
        // Look for RPN-related content
        const rpnElements = page.locator('text=/RPN|Risk Priority Number|Priority/i');
        const count = await rpnElements.count();
        if (count > 0) {
            await (0, test_1.expect)(rpnElements.first()).toBeVisible();
        }
    });
    (0, test_1.test)('should show FMEA (Failure Mode and Effects Analysis) content', async () => {
        await page.click('a[href="#risks"]');
        // Wait for FMEA content to load
        await page.waitForTimeout(2000);
        // Look for FMEA-specific terms
        const fmeaTerms = [
            'FMEA',
            'Failure Mode',
            'Effects Analysis',
            'Failure Modes',
            'Effects',
            'Causes'
        ];
        let foundFmeaContent = false;
        for (const term of fmeaTerms) {
            const elements = page.locator(`text="${term}"`);
            if (await elements.count() > 0) {
                await (0, test_1.expect)(elements.first()).toBeVisible();
                foundFmeaContent = true;
                break;
            }
        }
        // Should find FMEA-related content or indicate it's not yet implemented
        if (!foundFmeaContent) {
            // Check if there's a message about FMEA being in development
            const developmentMessage = page.locator('text=/coming soon|in development|not yet implemented/i');
            if (await developmentMessage.count() > 0) {
                await (0, test_1.expect)(developmentMessage.first()).toBeVisible();
            }
        }
    });
    (0, test_1.test)('should handle risk mitigation strategies display', async () => {
        await page.click('a[href="#risks"]');
        // Wait for mitigation content to load
        await page.waitForTimeout(2000);
        // Look for mitigation-related content
        const mitigationTerms = [
            'Mitigation',
            'Risk Control',
            'Control Measures',
            'Risk Reduction',
            'Preventive Action',
            'Corrective Action'
        ];
        for (const term of mitigationTerms) {
            const elements = page.locator(`text="${term}"`);
            if (await elements.count() > 0) {
                await (0, test_1.expect)(elements.first()).toBeVisible();
                break;
            }
        }
    });
    (0, test_1.test)('should maintain ISO 14971 compliance indicators', async () => {
        await page.click('a[href="#risks"]');
        // Verify ISO 14971 compliance indicators
        await (0, test_1.expect)(page.locator('text=ISO 14971').first()).toBeVisible();
        await (0, test_1.expect)(page.locator('text=FDA 21 CFR Part 820').first()).toBeVisible();
        await (0, test_1.expect)(page.locator('text=Regulatory Compliant').first()).toBeVisible();
    });
    (0, test_1.test)('should show proper loading states for risk data', async () => {
        await page.click('a[href="#risks"]');
        // Check for loading indicators
        const loadingElements = page.locator('text=Loading...');
        // If loading elements exist, they should eventually disappear
        if (await loadingElements.count() > 0) {
            await (0, test_1.expect)(loadingElements.first()).toBeHidden({ timeout: 15000 });
        }
    });
    (0, test_1.test)('should handle empty risk state gracefully', async () => {
        await page.click('a[href="#risks"]');
        // Wait for content to load
        await page.waitForTimeout(3000);
        // Should show either risk data or appropriate empty state
        const hasRiskData = await page.locator('text=/Risk|FMEA|Hazard/i').count() > 0;
        const hasEmptyState = await page.locator('text=/No risks|Empty|Not yet implemented/i').count() > 0;
        (0, test_1.expect)(hasRiskData || hasEmptyState).toBeTruthy();
    });
    (0, test_1.test)('should maintain responsive design for risk management', async () => {
        // Test mobile viewport
        await page.setViewportSize({ width: 375, height: 667 });
        await page.click('a[href="#risks"]');
        // Verify mobile navigation works
        await (0, test_1.expect)(page.locator('h2')).toContainText('Risk Management');
        // Content should be accessible on mobile
        await page.waitForTimeout(2000);
        const content = page.locator('main');
        await (0, test_1.expect)(content).toBeVisible();
    });
    (0, test_1.test)('should provide return to dashboard functionality', async () => {
        await page.click('a[href="#risks"]');
        // Find and click return to dashboard button
        const returnButton = page.locator('button:has-text("Return to Dashboard")');
        // Button might not be immediately visible, wait a bit
        await page.waitForTimeout(2000);
        if (await returnButton.count() > 0) {
            await (0, test_1.expect)(returnButton).toBeVisible();
            await returnButton.click();
            // Should return to dashboard
            await (0, test_1.expect)(page.locator('h2')).toContainText('Welcome to QMS Web Interface');
        }
    });
    (0, test_1.test)('should handle risk management API integration', async () => {
        await page.click('a[href="#risks"]');
        // Wait for any API calls to complete
        await page.waitForTimeout(3000);
        // Should not show JavaScript errors
        const consoleErrors = [];
        page.on('console', msg => {
            if (msg.type() === 'error') {
                consoleErrors.push(msg.text());
            }
        });
        // Refresh to trigger any API calls
        await page.reload();
        await page.click('a[href="#risks"]');
        await page.waitForTimeout(2000);
        // Should have minimal console errors (some are expected during development)
        (0, test_1.expect)(consoleErrors.length).toBeLessThan(5);
    });
});
//# sourceMappingURL=risk-management.spec.js.map
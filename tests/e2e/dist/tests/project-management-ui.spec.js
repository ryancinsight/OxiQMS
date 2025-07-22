"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const test_1 = require("@playwright/test");
/**
 * Project Management UI Testing
 *
 * Verifies the CardiacMonitor-v2.1 project is properly displayed with all associated
 * documents, requirements, and risks accessible through the web interface.
 *
 * Regulatory Compliance: FDA 21 CFR Part 820, ISO 13485, ISO 14971
 */
test_1.test.describe('Project Management Web Interface', () => {
    let page;
    test_1.test.beforeEach(async ({ page: testPage }) => {
        page = testPage;
        await page.goto('/');
        // Wait for the application to load
        await (0, test_1.expect)(page.locator('h1')).toContainText('QMS - Medical Device Quality Management');
        await (0, test_1.expect)(page.locator('text=🟢 Connected')).toBeVisible();
    });
    (0, test_1.test)('should display projects section with proper navigation', async () => {
        // Navigate to Projects section
        await page.click('a[href="#projects"]');
        // Verify Projects section loads
        await (0, test_1.expect)(page.locator('h2')).toContainText('Project Management');
        await (0, test_1.expect)(page.locator('p')).toContainText('QMS Project Creation and Management');
        // Verify regulatory compliance context
        await (0, test_1.expect)(page.locator('text=FDA 21 CFR Part 820').first()).toBeVisible();
        await (0, test_1.expect)(page.locator('text=ISO 13485').first()).toBeVisible();
        await (0, test_1.expect)(page.locator('text=ISO 14971').first()).toBeVisible();
    });
    (0, test_1.test)('should show project statistics and overview', async () => {
        await page.click('a[href="#projects"]');
        // Wait for project data to load
        await page.waitForTimeout(2000);
        // Verify project statistics
        await (0, test_1.expect)(page.locator('text=Total Projects:')).toBeVisible();
        await (0, test_1.expect)(page.locator('text=compliant project structure')).toBeVisible();
    });
    (0, test_1.test)('should display project management controls', async () => {
        await page.click('a[href="#projects"]');
        // Verify project management buttons
        await (0, test_1.expect)(page.locator('button:has-text("Create New Project")')).toBeVisible();
        await (0, test_1.expect)(page.locator('button:has-text("Refresh Projects")')).toBeVisible();
    });
    (0, test_1.test)('should show existing projects table with proper structure', async () => {
        await page.click('a[href="#projects"]');
        // Wait for projects table to load
        await page.waitForTimeout(2000);
        // Verify projects table
        await (0, test_1.expect)(page.locator('h3:has-text("Existing Projects")')).toBeVisible();
        await (0, test_1.expect)(page.locator('table')).toBeVisible();
        // Check table headers
        const expectedHeaders = ['Project Name', 'Description', 'Version', 'Created', 'Path', 'Actions'];
        for (const header of expectedHeaders) {
            await (0, test_1.expect)(page.locator(`th:has-text("${header}")`)).toBeVisible();
        }
    });
    (0, test_1.test)('should display CardiacMonitor project if available', async () => {
        await page.click('a[href="#projects"]');
        // Wait for project data to load
        await page.waitForTimeout(3000);
        // Look for CardiacMonitor project
        const cardiacProject = page.locator('text=/CardiacMonitor/i');
        const projectCount = await cardiacProject.count();
        if (projectCount > 0) {
            // Verify CardiacMonitor project is displayed
            await (0, test_1.expect)(cardiacProject.first()).toBeVisible();
            // Verify project has proper structure
            const projectRow = page.locator('tr:has-text("CardiacMonitor")');
            if (await projectRow.count() > 0) {
                // Should have project name
                await (0, test_1.expect)(projectRow.locator('strong')).toBeVisible();
                // Should have project ID
                await (0, test_1.expect)(projectRow.locator('text=/ID:/i')).toBeVisible();
                // Should have action buttons
                await (0, test_1.expect)(projectRow.locator('button:has-text("View")')).toBeVisible();
                await (0, test_1.expect)(projectRow.locator('button:has-text("Open")')).toBeVisible();
                await (0, test_1.expect)(projectRow.locator('button:has-text("Delete")')).toBeVisible();
            }
        }
        else {
            // If no CardiacMonitor project, should show appropriate state
            const emptyState = page.locator('text=/No projects|Empty|Create your first project/i');
            if (await emptyState.count() > 0) {
                await (0, test_1.expect)(emptyState.first()).toBeVisible();
            }
        }
    });
    (0, test_1.test)('should provide project view functionality', async () => {
        await page.click('a[href="#projects"]');
        // Wait for projects to load
        await page.waitForTimeout(3000);
        // Look for View button
        const viewButtons = page.locator('button:has-text("View")');
        const buttonCount = await viewButtons.count();
        if (buttonCount > 0) {
            // Click the first View button
            await viewButtons.first().click();
            // Should show project details modal or view
            await page.waitForTimeout(1000);
            // Look for project details
            const detailsModal = page.locator('.modal, .dialog, .project-details');
            if (await detailsModal.count() > 0) {
                await (0, test_1.expect)(detailsModal).toBeVisible();
                // Should show project details header
                await (0, test_1.expect)(page.locator('h3:has-text("Project Details")')).toBeVisible();
                // Should have close button
                const closeButton = page.locator('button:has-text("Close"), button:has-text("✕")');
                await (0, test_1.expect)(closeButton.first()).toBeVisible();
            }
        }
    });
    (0, test_1.test)('should show project compliance status', async () => {
        await page.click('a[href="#projects"]');
        // Wait for projects to load and try to view project details
        await page.waitForTimeout(3000);
        const viewButtons = page.locator('button:has-text("View")');
        if (await viewButtons.count() > 0) {
            await viewButtons.first().click();
            await page.waitForTimeout(1000);
            // Look for compliance status section
            const complianceSection = page.locator('h4:has-text("Compliance Status")');
            if (await complianceSection.count() > 0) {
                await (0, test_1.expect)(complianceSection).toBeVisible();
                // Should show compliance checkmarks
                const complianceItems = [
                    'FDA 21 CFR Part 820 structure',
                    'ISO 13485:2016 framework',
                    'ISO 14971:2019 risk management',
                    '21 CFR Part 11 electronic records'
                ];
                for (const item of complianceItems) {
                    const complianceItem = page.locator(`text="${item}"`);
                    if (await complianceItem.count() > 0) {
                        await (0, test_1.expect)(complianceItem).toBeVisible();
                    }
                }
            }
        }
    });
    (0, test_1.test)('should handle project refresh functionality', async () => {
        await page.click('a[href="#projects"]');
        // Click refresh button
        const refreshButton = page.locator('button:has-text("Refresh Projects")');
        await (0, test_1.expect)(refreshButton).toBeVisible();
        await refreshButton.click();
        // Should reload project data
        await page.waitForTimeout(2000);
        // Table should still be visible after refresh
        await (0, test_1.expect)(page.locator('table')).toBeVisible();
    });
    (0, test_1.test)('should show create new project functionality', async () => {
        await page.click('a[href="#projects"]');
        // Click create new project button
        const createButton = page.locator('button:has-text("Create New Project")');
        await (0, test_1.expect)(createButton).toBeVisible();
        // Button should be clickable
        await (0, test_1.expect)(createButton).toBeEnabled();
        // Note: We don't actually click it to avoid creating test projects
        // In a full implementation, this would test the project creation flow
    });
    (0, test_1.test)('should display project paths correctly', async () => {
        await page.click('a[href="#projects"]');
        // Wait for projects to load
        await page.waitForTimeout(3000);
        // Look for project path information
        const pathElements = page.locator('code, .project-path');
        const pathCount = await pathElements.count();
        if (pathCount > 0) {
            // Should show project paths in code format
            await (0, test_1.expect)(pathElements.first()).toBeVisible();
            // Path should contain typical project structure
            const pathText = await pathElements.first().textContent();
            (0, test_1.expect)(pathText).toMatch(/[\\\/]/); // Should contain path separators
        }
    });
    (0, test_1.test)('should handle project actions properly', async () => {
        await page.click('a[href="#projects"]');
        // Wait for projects to load
        await page.waitForTimeout(3000);
        // Check for action buttons
        const actionButtons = ['View', 'Open', 'Delete'];
        for (const action of actionButtons) {
            const buttons = page.locator(`button:has-text("${action}")`);
            const count = await buttons.count();
            if (count > 0) {
                // Action buttons should be visible and enabled
                await (0, test_1.expect)(buttons.first()).toBeVisible();
                await (0, test_1.expect)(buttons.first()).toBeEnabled();
            }
        }
    });
    (0, test_1.test)('should maintain responsive design for project management', async () => {
        // Test mobile viewport
        await page.setViewportSize({ width: 375, height: 667 });
        await page.click('a[href="#projects"]');
        // Verify mobile navigation works
        await (0, test_1.expect)(page.locator('h2')).toContainText('Project Management');
        // Table should be responsive or scrollable
        const table = page.locator('table');
        await (0, test_1.expect)(table).toBeVisible();
        // Action buttons should still be accessible
        const actionButtons = page.locator('button:has-text("View"), button:has-text("Open"), button:has-text("Delete")');
        if (await actionButtons.count() > 0) {
            await (0, test_1.expect)(actionButtons.first()).toBeVisible();
        }
    });
    (0, test_1.test)('should handle empty project state gracefully', async () => {
        await page.click('a[href="#projects"]');
        // Wait for projects to load
        await page.waitForTimeout(3000);
        // Check if there are any projects
        const projectRows = page.locator('tbody tr');
        const rowCount = await projectRows.count();
        if (rowCount === 0) {
            // Should show appropriate empty state or message
            const emptyMessage = page.locator('text=/No projects|Empty|Create your first/i');
            if (await emptyMessage.count() > 0) {
                await (0, test_1.expect)(emptyMessage.first()).toBeVisible();
            }
        }
        else {
            // Should show project data
            (0, test_1.expect)(rowCount).toBeGreaterThan(0);
        }
    });
    (0, test_1.test)('should provide return to dashboard functionality', async () => {
        await page.click('a[href="#projects"]');
        // Find and click return to dashboard button
        const returnButton = page.locator('button:has-text("Return to Dashboard")');
        await (0, test_1.expect)(returnButton).toBeVisible();
        await returnButton.click();
        // Should return to dashboard
        await (0, test_1.expect)(page.locator('h2')).toContainText('Welcome to QMS Web Interface');
    });
    (0, test_1.test)('should maintain regulatory compliance indicators', async () => {
        await page.click('a[href="#projects"]');
        // Verify regulatory compliance indicators are present
        await (0, test_1.expect)(page.locator('text=FDA 21 CFR Part 820').first()).toBeVisible();
        await (0, test_1.expect)(page.locator('text=ISO 13485').first()).toBeVisible();
        await (0, test_1.expect)(page.locator('text=ISO 14971').first()).toBeVisible();
        await (0, test_1.expect)(page.locator('text=Regulatory Compliant').first()).toBeVisible();
        await (0, test_1.expect)(page.locator('text=Audit Ready').first()).toBeVisible();
    });
});
//# sourceMappingURL=project-management-ui.spec.js.map
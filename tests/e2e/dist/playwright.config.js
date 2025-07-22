"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const test_1 = require("@playwright/test");
/**
 * QMS Medical Device Quality Management System - E2E Test Configuration
 * Regulatory Compliance: FDA 21 CFR Part 820, ISO 13485, ISO 14971
 *
 * This configuration ensures comprehensive testing of the QMS web interface
 * with focus on medical device regulatory compliance requirements.
 */
exports.default = (0, test_1.defineConfig)({
    testDir: './tests',
    /* Run tests in files in parallel */
    fullyParallel: true,
    /* Fail the build on CI if you accidentally left test.only in the source code. */
    forbidOnly: !!process.env.CI,
    /* Retry on CI only */
    retries: process.env.CI ? 2 : 0,
    /* Opt out of parallel tests on CI. */
    workers: process.env.CI ? 1 : undefined,
    /* Reporter to use. See https://playwright.dev/docs/test-reporters */
    reporter: [
        ['html', { outputFolder: 'playwright-report' }],
        ['json', { outputFile: 'test-results/results.json' }],
        ['junit', { outputFile: 'test-results/junit.xml' }],
        ['line']
    ],
    /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
    use: {
        /* Base URL to use in actions like `await page.goto('/')`. */
        baseURL: process.env.QMS_BASE_URL || 'http://localhost:8080',
        /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
        trace: 'on-first-retry',
        /* Take screenshot on failure */
        screenshot: 'only-on-failure',
        /* Record video on failure */
        video: 'retain-on-failure',
        /* Global timeout for each test */
        actionTimeout: 30000,
        navigationTimeout: 30000,
    },
    /* Configure projects for major browsers */
    projects: [
        {
            name: 'chromium',
            use: { ...test_1.devices['Desktop Chrome'] },
        },
        {
            name: 'firefox',
            use: { ...test_1.devices['Desktop Firefox'] },
        },
        {
            name: 'webkit',
            use: { ...test_1.devices['Desktop Safari'] },
        },
        /* Test against mobile viewports. */
        {
            name: 'Mobile Chrome',
            use: { ...test_1.devices['Pixel 5'] },
        },
        {
            name: 'Mobile Safari',
            use: { ...test_1.devices['iPhone 12'] },
        },
        /* Test against branded browsers. */
        {
            name: 'Microsoft Edge',
            use: { ...test_1.devices['Desktop Edge'], channel: 'msedge' },
        },
        {
            name: 'Google Chrome',
            use: { ...test_1.devices['Desktop Chrome'], channel: 'chrome' },
        },
    ],
    /* Run your local dev server before starting the tests */
    webServer: {
        command: 'cargo run -- serve --host 127.0.0.1 --port 8080',
        url: 'http://127.0.0.1:8080',
        reuseExistingServer: !process.env.CI,
        cwd: '../..',
        timeout: 120 * 1000,
        stdout: 'pipe',
        stderr: 'pipe',
    },
    /* Global setup and teardown - temporarily disabled for debugging */
    // globalSetup: require.resolve('./setup/global-setup.ts'),
    // globalTeardown: require.resolve('./setup/global-teardown.ts'),
    /* Test timeout */
    timeout: 60000,
    /* Expect timeout */
    expect: {
        timeout: 10000,
    },
});
//# sourceMappingURL=playwright.config.js.map
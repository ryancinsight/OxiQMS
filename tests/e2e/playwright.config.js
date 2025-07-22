// @ts-check
const { defineConfig, devices } = require('@playwright/test');

/**
 * QMS Medical Device Quality Management System - E2E Test Configuration
 * Regulatory Compliance: FDA 21 CFR Part 820, ISO 13485, ISO 14971
 */
module.exports = defineConfig({
  testDir: './tests',
  fullyParallel: false, // Sequential for medical device compliance testing
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1, // Single worker for consistent testing
  reporter: [
    ['html', { outputFolder: 'test-results/html-report' }],
    ['json', { outputFile: 'test-results/results.json' }],
    ['line']
  ],
  use: {
    baseURL: 'http://127.0.0.1:8080',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    actionTimeout: 30000,
    navigationTimeout: 30000,
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
    {
      name: 'Mobile Chrome',
      use: { ...devices['Pixel 5'] },
    },
  ],

  timeout: 60000,
  expect: {
    timeout: 10000,
  },
});

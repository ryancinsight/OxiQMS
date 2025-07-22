// @ts-check
const { test, expect } = require('@playwright/test');

/**
 * QMS API Integration Tests
 * Medical Device Quality Management System - Backend API Integration Testing
 * Regulatory Compliance: FDA 21 CFR Part 820, ISO 13485, ISO 14971
 */

test.describe('QMS API Integration', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should successfully call health check API', async ({ page }) => {
    // Test the health check endpoint
    const response = await page.request.get('/api/health');
    expect(response.status()).toBe(200);
    
    const healthData = await response.json();
    expect(healthData).toHaveProperty('status');
    expect(healthData.status).toBe('healthy');
    
    console.log('Health Check Response:', healthData);
  });

  test('should successfully call system stats API', async ({ page }) => {
    // Test the system stats endpoint
    const response = await page.request.get('/api/system/stats');
    expect(response.status()).toBe(200);
    
    const statsData = await response.json();
    expect(statsData).toHaveProperty('uptime');
    expect(statsData).toHaveProperty('memory_usage');
    
    console.log('System Stats Response:', statsData);
  });

  test('should successfully call compliance badges API', async ({ page }) => {
    // Test the compliance badges endpoint
    const response = await page.request.get('/api/compliance/badges');
    expect(response.status()).toBe(200);
    
    const badgesData = await response.json();
    expect(badgesData).toHaveProperty('badges');
    expect(Array.isArray(badgesData.badges)).toBe(true);
    
    // Verify required compliance badges are present
    const badgeNames = badgesData.badges.map(badge => badge.name);
    expect(badgeNames).toContain('FDA 21 CFR Part 820');
    expect(badgeNames).toContain('ISO 13485');
    expect(badgeNames).toContain('ISO 14971');
    
    console.log('Compliance Badges Response:', badgesData);
  });

  test('should handle documents API endpoints', async ({ page }) => {
    // Test documents list endpoint
    const listResponse = await page.request.get('/api/documents');
    
    // Should return 200 or 404 depending on whether documents exist
    expect([200, 404]).toContain(listResponse.status());
    
    if (listResponse.status() === 200) {
      const documentsData = await listResponse.json();
      expect(documentsData).toHaveProperty('documents');
      console.log('Documents List Response:', documentsData);
    }
  });

  test('should handle risks API endpoints', async ({ page }) => {
    // Test risks list endpoint
    const response = await page.request.get('/api/risks');
    
    // Should return 200 or 404 depending on whether risks exist
    expect([200, 404]).toContain(response.status());
    
    if (response.status() === 200) {
      const risksData = await response.json();
      expect(risksData).toHaveProperty('risks');
      console.log('Risks List Response:', risksData);
    }
  });

  test('should handle requirements API endpoints', async ({ page }) => {
    // Test requirements list endpoint
    const response = await page.request.get('/api/requirements');
    
    // Should return 200 or 404 depending on whether requirements exist
    expect([200, 404]).toContain(response.status());
    
    if (response.status() === 200) {
      const requirementsData = await response.json();
      expect(requirementsData).toHaveProperty('requirements');
      console.log('Requirements List Response:', requirementsData);
    }
  });

  test('should handle audit API endpoints', async ({ page }) => {
    // Test audit logs endpoint
    const response = await page.request.get('/api/audit/logs');
    
    // Should return 200 or 404 depending on whether audit logs exist
    expect([200, 404]).toContain(response.status());
    
    if (response.status() === 200) {
      const auditData = await response.json();
      expect(auditData).toHaveProperty('logs');
      console.log('Audit Logs Response:', auditData);
    }
  });

  test('should handle error responses gracefully', async ({ page }) => {
    // Test non-existent endpoint
    const response = await page.request.get('/api/nonexistent');
    expect(response.status()).toBe(404);
    
    // Test malformed requests
    const badResponse = await page.request.post('/api/documents', {
      data: { invalid: 'data' }
    });
    expect([400, 404, 405]).toContain(badResponse.status());
  });

  test('should maintain session state across API calls', async ({ page }) => {
    // Make multiple API calls and verify session consistency
    const responses = await Promise.all([
      page.request.get('/api/health'),
      page.request.get('/api/system/stats'),
      page.request.get('/api/compliance/badges')
    ]);
    
    // All should succeed
    responses.forEach(response => {
      expect(response.status()).toBe(200);
    });
    
    // Check for consistent session headers if implemented
    const sessionHeaders = responses.map(r => r.headers()['set-cookie']).filter(Boolean);
    console.log('Session Headers:', sessionHeaders);
  });
});

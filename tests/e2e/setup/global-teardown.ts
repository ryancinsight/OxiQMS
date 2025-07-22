import { FullConfig } from '@playwright/test';
import { execSync } from 'child_process';
import * as path from 'path';
import * as fs from 'fs';

/**
 * Global Teardown for QMS E2E Tests
 * 
 * Cleans up test data and ensures no test artifacts remain
 * Maintains audit trail integrity for regulatory compliance
 */
async function globalTeardown(config: FullConfig) {
  console.log('🧹 QMS E2E Test Teardown - Cleaning up test data');
  
  const qmsRoot = path.resolve(__dirname, '../../../');
  
  try {
    // Clean up test files
    const testFiles = ['test_srs.md', 'test_rmp.md', 'test_protocol.md'];
    testFiles.forEach(file => {
      const filePath = path.join(qmsRoot, file);
      if (fs.existsSync(filePath)) {
        fs.unlinkSync(filePath);
        console.log(`🗑️ Removed test file: ${file}`);
      }
    });
    
    // Clean up test project (optional - keep for debugging if needed)
    if (process.env.CLEANUP_TEST_PROJECT === 'true') {
      try {
        execSync('cargo run -- project delete --name "CardiacMonitor-E2E-Test" --force', {
          cwd: qmsRoot,
          stdio: 'pipe'
        });
        console.log('🗑️ Test project cleaned up');
      } catch (error) {
        console.log('ℹ️ Test project cleanup skipped (may not exist)');
      }
    }
    
    console.log('✅ Global teardown completed successfully');
    
  } catch (error) {
    console.error('❌ Global teardown failed:', error);
    // Don't throw - teardown failures shouldn't fail the test suite
  }
}

export default globalTeardown;

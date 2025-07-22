"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
const child_process_1 = require("child_process");
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
/**
 * Global Teardown for QMS E2E Tests
 *
 * Cleans up test data and ensures no test artifacts remain
 * Maintains audit trail integrity for regulatory compliance
 */
async function globalTeardown(config) {
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
                (0, child_process_1.execSync)('cargo run -- project delete --name "CardiacMonitor-E2E-Test" --force', {
                    cwd: qmsRoot,
                    stdio: 'pipe'
                });
                console.log('🗑️ Test project cleaned up');
            }
            catch (error) {
                console.log('ℹ️ Test project cleanup skipped (may not exist)');
            }
        }
        console.log('✅ Global teardown completed successfully');
    }
    catch (error) {
        console.error('❌ Global teardown failed:', error);
        // Don't throw - teardown failures shouldn't fail the test suite
    }
}
exports.default = globalTeardown;
//# sourceMappingURL=global-teardown.js.map
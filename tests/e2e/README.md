# QMS Comprehensive Web Browser Test Suite

## Overview

This comprehensive Playwright-based test suite **replaces the current backend database tests** with end-to-end web browser validation that ensures the QMS web interface provides complete functionality for medical device quality management without requiring CLI access.

## 🏥 Medical Device Compliance

- **FDA 21 CFR Part 820** - Quality System Regulation
- **ISO 13485:2016** - Medical devices - Quality management systems  
- **ISO 14971:2019** - Application of risk management to medical devices
- **21 CFR Part 11** - Electronic records; electronic signatures
- **Section 508** - Accessibility compliance
- **WCAG 2.1** - Web Content Accessibility Guidelines

## 🧪 Test Scope

### 1. Document Management Validation
- ✅ Verify CLI-created documents (SRS, Risk Management Plan, Test Protocol) display correctly
- ✅ Validate document metadata, content preview, and download functionality
- ✅ Test document version control and approval workflows
- ✅ Ensure regulatory compliance indicators are present

### 2. Requirements Tracking Verification  
- ✅ Confirm all requirements (REQ-001 through REQ-005) are visible in web interface
- ✅ Validate proper categorization (functional, safety, performance, regulatory)
- ✅ Test requirements status tracking and traceability
- ✅ Verify requirements matrix functionality

### 3. Risk Management Integration
- ✅ Validate risk assessments and FMEA entries created via CLI
- ✅ Test risk priority calculations (RPN) and severity levels
- ✅ Verify ISO 14971 compliance indicators
- ✅ Test risk mitigation tracking

### 4. Reports API Web Testing
- ✅ Automate compliance report generation through web interface
- ✅ Verify Audit Trail Report generation and content
- ✅ Test Risk Management Report generation per ISO 14971
- ✅ Validate Design History File Report functionality
- ✅ Test download and print functionality
- ✅ Ensure regulatory compliance standards in reports

### 5. Project Management UI Testing
- ✅ Verify CardiacMonitor-v2.1 project display
- ✅ Test project details modal and compliance status
- ✅ Validate project creation and management workflows
- ✅ Test project refresh and navigation functionality

### 6. Cross-Browser Compatibility
- ✅ Test Chrome, Firefox, Safari, and Edge browsers
- ✅ Validate mobile device compatibility (iOS/Android)
- ✅ Test responsive design across viewport sizes
- ✅ Verify accessibility compliance (keyboard navigation, screen readers)

### 7. End-to-End Workflow Validation
- ✅ Complete user journey testing without CLI dependency
- ✅ Validate all CRUD operations work through web interface
- ✅ Test audit trail logging for web-based operations
- ✅ Ensure regulatory compliance throughout workflows

## 🚀 Quick Start

### Prerequisites
- Node.js 18+ 
- npm or yarn
- QMS server running on localhost:8080

### Installation
```bash
cd tests/e2e
npm install
npx playwright install
```

### Run All Tests
```bash
# Linux/macOS
./run-comprehensive-tests.sh

# Windows
run-comprehensive-tests.bat

# Or manually
npm test
```

### Run Specific Test Suites
```bash
# Document management only
npm run test:chrome -- tests/document-management.spec.ts

# Requirements tracking only  
npm run test:chrome -- tests/requirements-tracking.spec.ts

# Reports API testing only
npm run test:chrome -- tests/reports-api-web.spec.ts

# Cross-browser testing
npm run test:cross-browser

# Mobile testing
npm run test:mobile
```

## 📊 Test Results and Reporting

### HTML Reports
```bash
npm run test:report
# Opens interactive HTML report at http://localhost:9323
```

### CI/CD Integration
```bash
# Headless testing for CI
npm test

# Generate JUnit XML for CI systems
npm test -- --reporter=junit
```

## 🏗️ Test Architecture

### Test Structure
```
tests/e2e/
├── setup/
│   ├── global-setup.ts          # Test data initialization
│   └── global-teardown.ts       # Cleanup
├── tests/
│   ├── document-management.spec.ts
│   ├── requirements-tracking.spec.ts  
│   ├── risk-management.spec.ts
│   ├── reports-api-web.spec.ts
│   ├── project-management-ui.spec.ts
│   ├── cross-browser-accessibility.spec.ts
│   └── end-to-end-workflow.spec.ts
├── playwright.config.ts         # Playwright configuration
├── package.json                 # Dependencies and scripts
└── README.md                    # This file
```

### Browser Matrix
- **Desktop**: Chrome, Firefox, Safari, Edge
- **Mobile**: Mobile Chrome (Android), Mobile Safari (iOS)
- **Tablets**: iPad, Android Tablet
- **Accessibility**: High contrast, reduced motion, keyboard-only

## 🎯 Success Criteria

### Functional Requirements
- [ ] All CLI-created data visible and manageable through web interface
- [ ] Complete QMS workflows possible without CLI access
- [ ] All CRUD operations functional through web UI
- [ ] File operations (view, download, preview) work reliably

### Regulatory Compliance
- [ ] FDA 21 CFR Part 820 compliance maintained throughout
- [ ] ISO 13485 and ISO 14971 indicators present
- [ ] Audit trail logging captures web-based operations
- [ ] Electronic records compliance (21 CFR Part 11)

### Technical Requirements  
- [ ] Cross-browser compatibility (Chrome, Firefox, Safari, Edge)
- [ ] Mobile responsiveness and accessibility
- [ ] Performance within acceptable limits (<10s load time)
- [ ] Visual consistency across browsers and devices

## 🔧 Configuration

### Environment Variables
```bash
# QMS server URL (default: http://localhost:8080)
export QMS_BASE_URL=http://localhost:8080

# Test timeout (default: 60000ms)
export PLAYWRIGHT_TIMEOUT=60000

# Cleanup test projects after tests
export CLEANUP_TEST_PROJECT=true
```

### Playwright Configuration
See `playwright.config.ts` for detailed browser and test configuration.

## 🐛 Troubleshooting

### Common Issues

**QMS Server Not Running**
```bash
# Start QMS server first
cd ../..
cargo run -- serve --port 8080
```

**Browser Installation Issues**
```bash
npx playwright install --with-deps
```

**TypeScript Compilation Errors**
```bash
npm run build
# Check tsconfig.json for configuration issues
```

**Test Timeouts**
- Increase timeout in `playwright.config.ts`
- Check QMS server performance
- Verify network connectivity

## 📈 Continuous Integration

### GitHub Actions Example
```yaml
name: QMS E2E Tests
on: [push, pull_request]
jobs:
  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
      - name: Install dependencies
        run: |
          cd tests/e2e
          npm install
          npx playwright install --with-deps
      - name: Run QMS E2E tests
        run: |
          cd tests/e2e
          npm test
      - name: Upload test results
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: playwright-report
          path: tests/e2e/playwright-report/
```

## 🏥 Medical Device Testing Standards

This test suite follows medical device software testing standards:
- **IEC 62304** - Medical device software lifecycle processes
- **ISO 14155** - Clinical investigation of medical devices
- **FDA Guidance** - Software as Medical Device (SaMD)

## 📞 Support

For issues with the test suite:
1. Check the troubleshooting section above
2. Review test logs in `test-results/`
3. Examine HTML reports for detailed failure information
4. Verify QMS server is running and accessible

---

**🎯 Goal**: Ensure users can perform complete QMS operations entirely through the web interface, making CLI optional while maintaining full medical device regulatory compliance.

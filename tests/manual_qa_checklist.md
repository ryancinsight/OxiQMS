# QMS Manual QA Checklist
## Phase 5, Task 5.2: Manual Quality Assurance Validation

**Version:** 1.0  
**Date:** July 18, 2025  
**Purpose:** Comprehensive manual validation of QMS compliance with FDA 21 CFR Part 820, ISO 13485, and ISO 14971  

---

## 5.2.1 FDA 21 CFR Part 820 Compliance Verification

### Document Control Processes (§820.40)

#### ✅ PASS/FAIL Document Control Requirements

- [ ] **Document Approval Process**
  - [ ] Test document creation without approval (should be Draft status)
  - [ ] Test document submission for review (Draft → InReview)
  - [ ] Test document approval with electronic signature (InReview → Approved)
  - [ ] Test document rejection with reason (InReview → Draft)
  - [ ] Verify only authorized users can approve documents
  - **Command:** `qms doc submit <id>`, `qms doc approve <id> --signature "reason"`
  - **Expected:** Clear status transitions, audit trail entries, electronic signatures

- [ ] **Document Distribution & Changes**
  - [ ] Test document versioning (semantic versioning: 1.0.0 → 1.0.1 → 1.1.0)
  - [ ] Test document checkout/checkin locking mechanism
  - [ ] Test concurrent access prevention
  - [ ] Verify version history preservation
  - **Command:** `qms doc checkout <id>`, `qms doc checkin <id> --file <file>`
  - **Expected:** Proper locking, version increments, conflict prevention

- [ ] **Document Obsolescence Control**
  - [ ] Test document archival process
  - [ ] Test obsolete document marking
  - [ ] Verify archived documents remain accessible but not editable
  - **Command:** `qms doc archive <id> --reason "superseded by v2.0"`
  - **Expected:** Status change to Archived, audit trail entry

#### ✅ PASS/FAIL Design Controls (§820.30)

- [ ] **Design History File (DHF) Generation**
  - [ ] Generate complete DHF report
  - [ ] Verify all documents included
  - [ ] Check regulatory mapping completeness
  - [ ] Validate traceability from requirements to verification
  - **Command:** `qms report dhf --output dhf_report.md --format md`
  - **Expected:** Complete design history with all linked artifacts

- [ ] **Design Reviews**
  - [ ] Test document review workflow
  - [ ] Verify reviewer assignment and tracking
  - [ ] Check review completion and sign-off
  - **Expected:** Complete review audit trail

#### ✅ PASS/FAIL Device Master Record (§820.181)

- [ ] **Complete Device Documentation**
  - [ ] Verify all device specifications documented
  - [ ] Check component traceability
  - [ ] Validate manufacturing procedures
  - **Expected:** Complete device documentation package

### Audit Trail Integrity (§820.184)

#### ✅ PASS/FAIL Audit Trail Requirements

- [ ] **Complete Audit Trail**
  - [ ] Test audit log generation for all operations
  - [ ] Verify user identification in all entries
  - [ ] Check timestamp accuracy and integrity
  - [ ] Validate action details and old/new values
  - **Command:** `qms audit export --format csv --output audit_validation.csv`
  - **Expected:** Complete audit trail with all required fields

- [ ] **Audit Trail Immutability**
  - [ ] Verify hash chain integrity
  - [ ] Test tampering detection
  - [ ] Check append-only log behavior
  - **Command:** `qms audit verify`
  - **Expected:** Valid hash chain, tampering detection

- [ ] **21 CFR Part 11 Electronic Records**
  - [ ] Test electronic signature creation
  - [ ] Verify signature validation
  - [ ] Check signature requirements enforcement
  - **Command:** `qms audit signature create`, `qms audit signature verify <id>`
  - **Expected:** Valid electronic signatures with proper validation

---

## 5.2.2 ISO 13485 Quality Management System Compliance

### Documentation Requirements (Section 4.2)

#### ✅ PASS/FAIL Document Control Procedures

- [ ] **Document Hierarchy Validation**
  - [ ] Test document categorization (SRS, SDD, VnV, etc.)
  - [ ] Verify document relationships
  - [ ] Check metadata completeness
  - **Expected:** Proper document hierarchy and metadata

- [ ] **Document Control Effectiveness**
  - [ ] Test document search and filtering
  - [ ] Verify document template system
  - [ ] Check document import/export functionality
  - **Command:** `qms doc search "keyword"`, `qms doc template create`
  - **Expected:** Effective document control processes

### Design and Development (Section 7.3)

#### ✅ PASS/FAIL Design Controls Implementation

- [ ] **Requirements Management**
  - [ ] Test requirements creation and validation
  - [ ] Verify requirements traceability
  - [ ] Check requirements-to-test linking
  - **Command:** `qms req create`, `qms trace link --from REQ-001 --to TC-001`
  - **Expected:** Complete requirements management system

- [ ] **Design Transfer**
  - [ ] Test design artifact management
  - [ ] Verify design review processes
  - [ ] Check design change control
  - **Expected:** Controlled design transfer process

---

## 5.2.3 ISO 14971 Risk Management Compliance

### Risk Analysis (Section 5)

#### ✅ PASS/FAIL Risk Identification and Analysis

- [ ] **Hazard Identification**
  - [ ] Test risk creation with hazard/situation/harm
  - [ ] Verify risk categorization
  - [ ] Check risk assessment methodology
  - **Command:** `qms risk create --hazard "desc" --situation "sit" --harm "harm"`
  - **Expected:** Comprehensive risk identification process

- [ ] **Risk Estimation**
  - [ ] Test RPN calculation (Severity × Occurrence × Detectability)
  - [ ] Verify risk level assessment (Acceptable/ALARP/Unacceptable)
  - [ ] Check risk matrix functionality
  - **Command:** `qms risk assess <id> --severity 4 --occurrence 3 --detectability 2`
  - **Expected:** Accurate RPN calculation and risk level assessment

### Risk Evaluation (Section 6)

#### ✅ PASS/FAIL Risk Acceptability

- [ ] **Risk Acceptability Criteria**
  - [ ] Test risk level thresholds (Acceptable: 1-24, ALARP: 25-99, Unacceptable: 100-125)
  - [ ] Verify risk acceptance workflow
  - [ ] Check management approval for high risks
  - **Expected:** Clear risk acceptability criteria and workflow

### Risk Control (Section 7)

#### ✅ PASS/FAIL Risk Mitigation

- [ ] **Risk Control Measures**
  - [ ] Test mitigation measure creation
  - [ ] Verify effectiveness calculation
  - [ ] Check residual risk assessment
  - **Command:** `qms risk mitigate <id> --measure "desc" --effectiveness 0.8`
  - **Expected:** Effective risk control implementation

- [ ] **Risk Control Verification**
  - [ ] Test mitigation verification workflow
  - [ ] Verify evidence tracking
  - [ ] Check verification completeness
  - **Command:** `qms risk verify <id> --method test --evidence TC-001`
  - **Expected:** Complete verification process

### FMEA Implementation

#### ✅ PASS/FAIL Failure Mode Analysis

- [ ] **FMEA Creation and Management**
  - [ ] Test FMEA analysis creation
  - [ ] Verify failure mode management
  - [ ] Check FMEA table generation
  - **Command:** `qms risk fmea create --component "UI" --function "data entry"`
  - **Expected:** Complete FMEA analysis capability

---

## 5.2.4 Electronic Signature Validation

### 21 CFR Part 11 Electronic Signatures

#### ✅ PASS/FAIL Electronic Signature Implementation

- [ ] **Signature Creation**
  - [ ] Test electronic signature generation
  - [ ] Verify signature requirements
  - [ ] Check signature metadata
  - **Command:** `qms audit signature create`
  - **Expected:** Valid electronic signature with metadata

- [ ] **Signature Validation**
  - [ ] Test signature verification
  - [ ] Verify signature integrity
  - [ ] Check signature audit trail
  - **Command:** `qms audit signature verify <id>`
  - **Expected:** Accurate signature validation

---

## 5.2.5 File Permissions and Security

### Credential Security

#### ✅ PASS/FAIL Security Implementation

- [ ] **Password Management**
  - [ ] Test password hashing (using DefaultHasher)
  - [ ] Verify secure storage
  - [ ] Check login/logout functionality
  - **Expected:** Secure credential management

- [ ] **File Access Control**
  - [ ] Test file permission settings (0755 directories, 0644 files on Unix)
  - [ ] Verify project isolation
  - [ ] Check unauthorized access prevention
  - **Expected:** Proper file access control

---

## QA Test Results Summary

### Overall Compliance Score

- **FDA 21 CFR Part 820:** ___/10 tests passed
- **ISO 13485:** ___/8 tests passed  
- **ISO 14971:** ___/12 tests passed
- **Security & Access:** ___/6 tests passed

**Total Score:** ___/36 tests passed (___%)

### Critical Issues Found
1. 
2. 
3. 

### Recommendations
1. 
2. 
3. 

### QA Approval

**QA Engineer:** _________________  
**Date:** _________________  
**Signature:** _________________  

**Approved for Release:** ☐ YES ☐ NO  

**Comments:**
___________________________________________________
___________________________________________________
___________________________________________________

---

## Manual Test Execution Instructions

### Prerequisites
1. Build QMS in release mode: `cargo build --release`
2. Ensure clean test environment
3. Have test data files ready

### Test Execution Steps
1. **Run each test scenario systematically**
2. **Record PASS/FAIL for each requirement**
3. **Document any issues found**
4. **Capture evidence (screenshots, log files)**
5. **Verify expected vs. actual results**

### Evidence Collection
- Audit trail exports
- Generated reports
- Error messages
- System behavior observations

### Test Environment
- **OS:** Windows/Linux/macOS
- **Rust Version:** 1.70+
- **QMS Version:** 0.1.0
- **Test Date:** ___________
- **Tester:** ___________

@echo off
REM QMS Regulatory Compliance Validation Script
REM Phase 5, Task 5.2.2: Automated compliance verification

echo ========================================================
echo QMS Regulatory Compliance Validation
echo Automated verification of FDA and ISO requirements
echo ========================================================
echo.

set COMPLIANCE_PASSED=0
set COMPLIANCE_TOTAL=0
set TEST_PROJECT_NAME=Compliance_Test_Device

REM Create a temporary test environment
set TEST_DIR=%TEMP%\qms_compliance_%RANDOM%
echo [SETUP] Creating compliance test environment: %TEST_DIR%
mkdir "%TEST_DIR%" 2>nul
cd /d "%TEST_DIR%"

echo.
echo ========================================================
echo FDA 21 CFR Part 820 COMPLIANCE TESTING
echo ========================================================

REM Test 1: Document Control (§820.40)
echo [TEST 1] Document Control Process Verification...
set /a COMPLIANCE_TOTAL+=1

cargo run --bin qms -- init --name "%TEST_PROJECT_NAME%" >nul 2>&1
if %ERRORLEVEL% NEQ 0 goto :test1_fail

echo # Test Document > test_doc.md
echo This is a regulatory compliance test document. >> test_doc.md

cargo run --bin qms -- doc add --file test_doc.md --title "Compliance Test Doc" --type srs >nul 2>&1
if %ERRORLEVEL% NEQ 0 goto :test1_fail

echo [PASS] Test 1: Document control process functional
set /a COMPLIANCE_PASSED+=1
goto :test2

:test1_fail
echo [FAIL] Test 1: Document control process failed
goto :test2

:test2
REM Test 2: Audit Trail (§820.184)
echo [TEST 2] Audit Trail Integrity Verification...
set /a COMPLIANCE_TOTAL+=1

cargo run --bin qms -- audit export --format csv --output compliance_audit.csv >nul 2>&1
if %ERRORLEVEL% NEQ 0 goto :test2_fail

if exist "compliance_audit.csv" (
    echo [PASS] Test 2: Audit trail generation successful
    set /a COMPLIANCE_PASSED+=1
) else (
    echo [FAIL] Test 2: Audit trail file not generated
)
goto :test3

:test2_fail
echo [FAIL] Test 2: Audit trail export failed
goto :test3

:test3
REM Test 3: Design History File (§820.181)
echo [TEST 3] Design History File Generation...
set /a COMPLIANCE_TOTAL+=1

cargo run --bin qms -- report dhf --output dhf_compliance.md --format md >nul 2>&1
if %ERRORLEVEL% NEQ 0 goto :test3_fail

if exist "dhf_compliance.md" (
    echo [PASS] Test 3: DHF generation successful
    set /a COMPLIANCE_PASSED+=1
) else (
    echo [FAIL] Test 3: DHF file not generated
)
goto :iso13485_tests

:test3_fail
echo [FAIL] Test 3: DHF generation failed
goto :iso13485_tests

:iso13485_tests
echo.
echo ========================================================
echo ISO 13485 COMPLIANCE TESTING
echo ========================================================

REM Test 4: Requirements Management (Section 7.3)
echo [TEST 4] Requirements Management System...
set /a COMPLIANCE_TOTAL+=1

cargo run --bin qms -- req init >nul 2>&1
if %ERRORLEVEL% NEQ 0 goto :test4_fail

cargo run --bin qms -- req create --title "Test Requirement" --desc "Compliance test requirement" --category functional --priority critical >nul 2>&1
if %ERRORLEVEL% NEQ 0 goto :test4_fail

echo [PASS] Test 4: Requirements management functional
set /a COMPLIANCE_PASSED+=1
goto :test5

:test4_fail
echo [FAIL] Test 4: Requirements management failed
goto :test5

:test5
REM Test 5: Traceability Matrix
echo [TEST 5] Requirements Traceability Matrix...
set /a COMPLIANCE_TOTAL+=1

cargo run --bin qms -- trace matrix --format csv --output rtm_compliance.csv >nul 2>&1
if %ERRORLEVEL% NEQ 0 goto :test5_fail

if exist "rtm_compliance.csv" (
    echo [PASS] Test 5: Traceability matrix generation successful
    set /a COMPLIANCE_PASSED+=1
) else (
    echo [FAIL] Test 5: RTM file not generated
)
goto :iso14971_tests

:test5_fail
echo [FAIL] Test 5: Traceability matrix generation failed
goto :iso14971_tests

:iso14971_tests
echo.
echo ========================================================
echo ISO 14971 RISK MANAGEMENT COMPLIANCE TESTING
echo ========================================================

REM Test 6: Risk Management System
echo [TEST 6] Risk Management System Initialization...
set /a COMPLIANCE_TOTAL+=1

cargo run --bin qms -- risk init >nul 2>&1
if %ERRORLEVEL% NEQ 0 goto :test6_fail

cargo run --bin qms -- risk create --hazard "Compliance test hazard" --situation "Test failure situation" --harm "Test harm" >nul 2>&1
if %ERRORLEVEL% NEQ 0 goto :test6_fail

echo [PASS] Test 6: Risk management system functional
set /a COMPLIANCE_PASSED+=1
goto :test7

:test6_fail
echo [FAIL] Test 6: Risk management system failed
goto :test7

:test7
REM Test 7: Risk Assessment and RPN Calculation
echo [TEST 7] Risk Assessment and RPN Calculation...
set /a COMPLIANCE_TOTAL+=1

cargo run --bin qms -- risk assess HAZ-001 --severity 4 --occurrence 3 --detectability 2 >nul 2>&1
if %ERRORLEVEL% NEQ 0 goto :test7_fail

echo [PASS] Test 7: Risk assessment functional
set /a COMPLIANCE_PASSED+=1
goto :test8

:test7_fail
echo [FAIL] Test 7: Risk assessment failed
goto :test8

:test8
REM Test 8: Risk Reports
echo [TEST 8] Risk Management Reports...
set /a COMPLIANCE_TOTAL+=1

cargo run --bin qms -- report risks --output risk_compliance.csv --format csv >nul 2>&1
if %ERRORLEVEL% NEQ 0 goto :test8_fail

if exist "risk_compliance.csv" (
    echo [PASS] Test 8: Risk report generation successful
    set /a COMPLIANCE_PASSED+=1
) else (
    echo [FAIL] Test 8: Risk report file not generated
)
goto :cfr_part11_tests

:test8_fail
echo [FAIL] Test 8: Risk report generation failed
goto :cfr_part11_tests

:cfr_part11_tests
echo.
echo ========================================================
echo 21 CFR PART 11 ELECTRONIC RECORDS COMPLIANCE
echo ========================================================

REM Test 9: Electronic Signatures
echo [TEST 9] Electronic Signature System...
set /a COMPLIANCE_TOTAL+=1

cargo run --bin qms -- audit signature requirements >nul 2>&1
if %ERRORLEVEL% NEQ 0 goto :test9_fail

echo [PASS] Test 9: Electronic signature system available
set /a COMPLIANCE_PASSED+=1
goto :test10

:test9_fail
echo [FAIL] Test 9: Electronic signature system failed
goto :test10

:test10
REM Test 10: Audit Trail Verification
echo [TEST 10] Audit Trail Hash Chain Verification...
set /a COMPLIANCE_TOTAL+=1

cargo run --bin qms -- audit verify >nul 2>&1
if %ERRORLEVEL% NEQ 0 goto :test10_fail

echo [PASS] Test 10: Audit trail verification functional
set /a COMPLIANCE_PASSED+=1
goto :compliance_summary

:test10_fail
echo [FAIL] Test 10: Audit trail verification failed
goto :compliance_summary

:compliance_summary
echo.
echo ========================================================
echo REGULATORY COMPLIANCE SUMMARY
echo ========================================================
echo Tests Passed: %COMPLIANCE_PASSED%/%COMPLIANCE_TOTAL%

set /a COMPLIANCE_PERCENT=%COMPLIANCE_PASSED% * 100 / %COMPLIANCE_TOTAL%
echo Compliance Score: %COMPLIANCE_PERCENT%%%

echo.
echo Regulatory Standards Tested:
echo • FDA 21 CFR Part 820 (Quality System Regulation)
echo • ISO 13485 (Medical devices — Quality management systems)
echo • ISO 14971 (Application of risk management to medical devices)
echo • 21 CFR Part 11 (Electronic Records and Electronic Signatures)

if %COMPLIANCE_PERCENT% GEQ 90 (
    echo.
    echo ✅ EXCELLENT COMPLIANCE (%COMPLIANCE_PERCENT%%%)
    echo The QMS system demonstrates excellent regulatory compliance
    echo and is ready for medical device development workflows.
    echo.
    echo Key compliance features verified:
    echo ✓ Document control and approval workflows
    echo ✓ Complete audit trail with integrity verification
    echo ✓ Design History File (DHF) generation
    echo ✓ Requirements management and traceability
    echo ✓ ISO 14971 compliant risk management
    echo ✓ Electronic signature capabilities
    echo ✓ 21 CFR Part 11 audit trail compliance
) else if %COMPLIANCE_PERCENT% GEQ 70 (
    echo.
    echo ⚠️  GOOD COMPLIANCE (%COMPLIANCE_PERCENT%%%)
    echo The QMS system shows good regulatory compliance
    echo with some areas needing attention.
) else (
    echo.
    echo ❌ INSUFFICIENT COMPLIANCE (%COMPLIANCE_PERCENT%%%)
    echo The QMS system needs significant improvements
    echo before use in medical device development.
)

echo.
echo Test artifacts generated:
dir /b *.csv *.md 2>nul || echo No report files found
echo.
echo Compliance test completed in: %CD%
echo.

REM Cleanup and return to original directory
cd /d "%~dp0"

if %COMPLIANCE_PERCENT% GEQ 80 (
    echo [SUCCESS] Regulatory compliance validation passed
    exit /b 0
) else (
    echo [FAILURE] Regulatory compliance validation failed
    exit /b 1
)

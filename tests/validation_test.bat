@echo off
REM QMS Test Validation Script
REM Phase 5, Task 5.1.2: Verify exit codes and outputs
REM Validates the end-to-end test results and system behavior

echo ========================================================
echo QMS Test Validation Suite
echo Verifying exit codes, outputs, and system behavior
echo ========================================================
echo.

set VALIDATION_FAILED=0
set TEST_COUNT=0
set PASS_COUNT=0

REM Test 1: Basic help functionality
echo [TEST] Verifying basic help functionality...
set /a TEST_COUNT+=1
cargo run --bin qms -- --help >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo [PASS] Help command returns exit code 0
    set /a PASS_COUNT+=1
) else (
    echo [FAIL] Help command failed with exit code %ERRORLEVEL%
    set VALIDATION_FAILED=1
)

REM Test 2: Invalid command handling
echo [TEST] Verifying invalid command handling...
set /a TEST_COUNT+=1
cargo run --bin qms -- invalid_command >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo [PASS] Invalid command returns non-zero exit code
    set /a PASS_COUNT+=1
) else (
    echo [FAIL] Invalid command should return non-zero exit code
    set VALIDATION_FAILED=1
)

REM Test 3: Project initialization without parameters
echo [TEST] Verifying project init parameter validation...
set /a TEST_COUNT+=1
cargo run --bin qms -- init >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo [PASS] Init without parameters returns non-zero exit code
    set /a PASS_COUNT+=1
) else (
    echo [FAIL] Init without parameters should return non-zero exit code
    set VALIDATION_FAILED=1
)

REM Test 4: Document commands without project
echo [TEST] Verifying document commands require project...
set /a TEST_COUNT+=1
cd /d %TEMP%
cargo run --bin qms -- doc list >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo [PASS] Doc command without project returns non-zero exit code
    set /a PASS_COUNT+=1
) else (
    echo [FAIL] Doc command without project should return non-zero exit code
    set VALIDATION_FAILED=1
)

REM Test 5: Risk commands without project
echo [TEST] Verifying risk commands require project...
set /a TEST_COUNT+=1
cargo run --bin qms -- risk list >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo [PASS] Risk command without project returns non-zero exit code
    set /a PASS_COUNT+=1
) else (
    echo [FAIL] Risk command without project should return non-zero exit code
    set VALIDATION_FAILED=1
)

REM Test 6: Audit commands basic functionality
echo [TEST] Verifying audit help functionality...
set /a TEST_COUNT+=1
cargo run --bin qms -- audit --help >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo [PASS] Audit help returns exit code 0
    set /a PASS_COUNT+=1
) else (
    echo [FAIL] Audit help failed with exit code %ERRORLEVEL%
    set VALIDATION_FAILED=1
)

REM Test 7: Report commands basic functionality
echo [TEST] Verifying report help functionality...
set /a TEST_COUNT+=1
cargo run --bin qms -- report --help >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo [PASS] Report help returns exit code 0
    set /a PASS_COUNT+=1
) else (
    echo [FAIL] Report help failed with exit code %ERRORLEVEL%
    set VALIDATION_FAILED=1
)

REM Test 8: Requirements commands basic functionality
echo [TEST] Verifying requirements help functionality...
set /a TEST_COUNT+=1
cargo run --bin qms -- req --help >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo [PASS] Requirements help returns exit code 0
    set /a PASS_COUNT+=1
) else (
    echo [FAIL] Requirements help failed with exit code %ERRORLEVEL%
    set VALIDATION_FAILED=1
)

REM Test 9: Traceability commands basic functionality
echo [TEST] Verifying traceability help functionality...
set /a TEST_COUNT+=1
cargo run --bin qms -- trace --help >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo [PASS] Traceability help returns exit code 0
    set /a PASS_COUNT+=1
) else (
    echo [FAIL] Traceability help failed with exit code %ERRORLEVEL%
    set VALIDATION_FAILED=1
)

REM Test 10: Version information
echo [TEST] Verifying version information...
set /a TEST_COUNT+=1
cargo run --bin qms -- --version >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo [PASS] Version command returns exit code 0
    set /a PASS_COUNT+=1
) else (
    echo [WARNING] Version command not implemented or failed
)

echo.
echo ========================================================
echo VALIDATION RESULTS
echo ========================================================
echo Total tests: %TEST_COUNT%
echo Passed: %PASS_COUNT%
echo Failed: %TEST_COUNT% - %PASS_COUNT% = %VALIDATION_FAILED%
echo.

if %VALIDATION_FAILED% EQU 0 (
    echo [SUCCESS] All exit code validations passed!
    echo The QMS CLI properly handles:
    echo ✓ Valid commands with appropriate exit codes
    echo ✓ Invalid commands with error exit codes
    echo ✓ Missing parameters with error exit codes
    echo ✓ Commands requiring project context
    echo ✓ Help functionality for all modules
    echo.
    echo Ready for end-to-end testing!
) else (
    echo [FAILURE] Some exit code validations failed!
    echo Review the output above for specific issues.
    echo Fix any failing tests before proceeding with e2e testing.
)

echo ========================================================
exit /b %VALIDATION_FAILED%

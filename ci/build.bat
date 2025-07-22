@echo off
REM QMS CI Build Script for Windows
REM Medical Device Quality Management System
REM This script performs comprehensive testing and validation

echo =============================================
echo QMS Medical Device Quality Management System
echo CI Build Pipeline for Windows
echo =============================================

REM Set error handling
setlocal enabledelayedexpansion
set ERROR_LEVEL=0

echo.
echo [Step 1/6] Checking Rust installation...
rustc --version >nul 2>&1
if !errorlevel! neq 0 (
    echo ERROR: Rust is not installed or not in PATH
    set ERROR_LEVEL=1
    goto :error
)
rustc --version
echo ✓ Rust installation verified

echo.
echo [Step 2/6] Code formatting check...
cargo fmt -- --check
if !errorlevel! neq 0 (
    echo ERROR: Code formatting issues detected. Run 'cargo fmt' to fix.
    set ERROR_LEVEL=1
    goto :error
)
echo ✓ Code formatting check passed

echo.
echo [Step 3/6] Linting with Clippy...
cargo clippy --all-targets --all-features -- -D clippy::correctness -D clippy::suspicious -A clippy::style -A clippy::complexity -A clippy::perf
if !errorlevel! neq 0 (
    echo ERROR: Clippy critical linting failed. Fix issues before proceeding.
    set ERROR_LEVEL=1
    goto :error
)
echo ✓ Clippy linting passed

echo.
echo [Step 4/6] Running unit tests...
cargo test --bin qms
if !errorlevel! neq 0 (
    echo ERROR: Unit tests failed
    set ERROR_LEVEL=1
    goto :error
)
echo ✓ Unit tests passed

echo.
echo [Step 5/6] Running integration tests...
cargo test --tests
if !errorlevel! neq 0 (
    echo ERROR: Integration tests failed
    set ERROR_LEVEL=1
    goto :error
)
echo ✓ Integration tests passed

echo.
echo [Step 6/6] Security audit check...
REM Check for common security issues in dependencies
REM Since we're stdlib-only, this mainly checks for unsafe code usage
findstr /i /c:"unsafe" src\*.rs >nul 2>&1
if !errorlevel! equ 0 (
    echo WARNING: Unsafe code detected. Review for regulatory compliance.
    echo Listing unsafe code locations:
    findstr /i /n /c:"unsafe" src\*.rs
)

REM Check for unwrap calls which could cause panics
findstr /i /c:".unwrap(" src\*.rs >nul 2>&1
if !errorlevel! equ 0 (
    echo WARNING: unwrap calls detected. Consider using proper error handling.
    echo Listing unwrap locations:
    findstr /i /n /c:".unwrap(" src\*.rs
)

echo ✓ Security audit completed

echo.
echo =============================================
echo BUILD SUCCESSFUL
echo All checks passed. Ready for deployment.
echo =============================================
goto :end

:error
echo.
echo =============================================
echo BUILD FAILED
echo Error level: !ERROR_LEVEL!
echo Please fix the above issues and retry.
echo =============================================
exit /b !ERROR_LEVEL!

:end
endlocal

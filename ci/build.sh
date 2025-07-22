#!/bin/bash
# QMS CI Build Script for Unix/Linux/macOS
# Medical Device Quality Management System
# This script performs comprehensive testing and validation

set -e  # Exit on any error

echo "============================================="
echo "QMS Medical Device Quality Management System"
echo "CI Build Pipeline for Unix/Linux/macOS"
echo "============================================="

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

error_exit() {
    echo -e "${RED}ERROR: $1${NC}" >&2
    exit 1
}

warning() {
    echo -e "${YELLOW}WARNING: $1${NC}"
}

success() {
    echo -e "${GREEN}✓ $1${NC}"
}

echo
echo "[Step 1/6] Checking Rust installation..."
if ! command -v rustc &> /dev/null; then
    error_exit "Rust is not installed or not in PATH"
fi
rustc --version
success "Rust installation verified"

echo
echo "[Step 2/6] Code formatting check..."
if ! cargo fmt -- --check; then
    error_exit "Code formatting issues detected. Run 'cargo fmt' to fix."
fi
success "Code formatting check passed"

echo
echo "[Step 3/6] Linting with Clippy..."
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    error_exit "Clippy linting failed. Fix warnings before proceeding."
fi
success "Clippy linting passed"

echo
echo "[Step 4/6] Running unit tests..."
if ! cargo test --lib; then
    error_exit "Unit tests failed"
fi
success "Unit tests passed"

echo
echo "[Step 5/6] Running integration tests..."
if ! cargo test --test '*'; then
    error_exit "Integration tests failed"
fi
success "Integration tests passed"

echo
echo "[Step 6/6] Security audit check..."
# Check for common security issues
# Since we're stdlib-only, this mainly checks for unsafe code usage
if grep -r "unsafe" src/ >/dev/null 2>&1; then
    warning "Unsafe code detected. Review for regulatory compliance."
    echo "Listing unsafe code locations:"
    grep -rn "unsafe" src/ || true
fi

# Check for unwrap() usage which could cause panics
if grep -r "\.unwrap()" src/ >/dev/null 2>&1; then
    warning "unwrap() calls detected. Consider using proper error handling."
    echo "Listing unwrap() locations:"
    grep -rn "\.unwrap()" src/ || true
fi

success "Security audit completed"

echo
echo "============================================="
echo "BUILD SUCCESSFUL"
echo "All checks passed. Ready for deployment."
echo "============================================="

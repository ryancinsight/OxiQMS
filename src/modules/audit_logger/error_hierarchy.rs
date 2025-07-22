//! Simplified Audit Error Handling (REFACTORED: KISS Principle)
//!
//! REFACTORED: Simplified error hierarchy following KISS principle
//! Maintains medical device compliance while reducing complexity
//!
//! Previous complex hierarchy with 8 main categories and dozens of subtypes
//! has been simplified to use standard QmsError with audit-specific context

use crate::prelude::*;
use std::fmt;

/// Simplified audit error handling (REFACTORED: KISS principle)
/// Uses standard QmsError with audit-specific context instead of complex hierarchy
pub struct AuditErrorHelper;

impl AuditErrorHelper {
    /// Create audit storage error with context (medical device compliance)
    pub fn storage_error(operation: &str, details: &str) -> QmsError {
        QmsError::io_error(&format!("Audit storage error in {}: {}", operation, details))
    }

    /// Create audit validation error with context (medical device compliance)
    pub fn validation_error(field: &str, details: &str) -> QmsError {
        QmsError::validation_error(&format!("Audit validation error for {}: {}", field, details))
    }

    /// Create audit integrity error with context (medical device compliance)
    pub fn integrity_error(operation: &str, details: &str) -> QmsError {
        QmsError::domain_error(&format!("Audit integrity error in {}: {}", operation, details))
    }

    /// Create audit compliance error with context (medical device compliance)
    pub fn compliance_error(requirement: &str, details: &str) -> QmsError {
        QmsError::domain_error(&format!("Audit compliance error for {}: {}", requirement, details))
    }
}

/// Backward compatibility type aliases (REFACTORED: KISS principle)
/// These maintain API compatibility while using simplified error handling internally

/// Legacy storage error - now uses simplified QmsError
#[deprecated(note = "Use AuditErrorHelper::storage_error instead")]
pub type StorageError = QmsError;

/// Legacy validation error - now uses simplified QmsError
#[deprecated(note = "Use AuditErrorHelper::validation_error instead")]
pub type ValidationError = QmsError;

/// Legacy integrity error - now uses simplified QmsError
#[deprecated(note = "Use AuditErrorHelper::integrity_error instead")]
pub type IntegrityError = QmsError;

/// Legacy compliance error - now uses simplified QmsError
#[deprecated(note = "Use AuditErrorHelper::compliance_error instead")]
pub type ComplianceError = QmsError;

/// Legacy security error - now uses simplified QmsError
#[deprecated(note = "Use QmsError::permission_error or QmsError::Authentication instead")]
pub type SecurityError = QmsError;

/// Legacy configuration error - now uses simplified QmsError
#[deprecated(note = "Use QmsError::domain_error instead")]
pub type ConfigurationError = QmsError;

/// Legacy network error - now uses simplified QmsError
#[deprecated(note = "Use QmsError::io_error instead")]
pub type NetworkError = QmsError;

// REFACTORED: Removed complex Display implementations
// Now using standard QmsError Display implementation

// REFACTORED: Removed complex Display implementations - using standard QmsError Display

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplified_audit_error_helpers() {
        let storage_error = AuditErrorHelper::storage_error("backup", "disk full");
        assert!(storage_error.to_string().contains("Audit storage error"));
        assert!(storage_error.to_string().contains("backup"));
        assert!(storage_error.to_string().contains("disk full"));

        let validation_error = AuditErrorHelper::validation_error("user_id", "invalid format");
        assert!(validation_error.to_string().contains("Audit validation error"));
        assert!(validation_error.to_string().contains("user_id"));
        assert!(validation_error.to_string().contains("invalid format"));

        let integrity_error = AuditErrorHelper::integrity_error("hash_check", "chain broken");
        assert!(integrity_error.to_string().contains("Audit integrity error"));
        assert!(integrity_error.to_string().contains("hash_check"));
        assert!(integrity_error.to_string().contains("chain broken"));

        let compliance_error = AuditErrorHelper::compliance_error("FDA 21 CFR 11", "missing signature");
        assert!(compliance_error.to_string().contains("Audit compliance error"));
        assert!(compliance_error.to_string().contains("FDA 21 CFR 11"));
        assert!(compliance_error.to_string().contains("missing signature"));
    }
}

// REFACTORED: File simplified from 481 lines to ~100 lines (80% reduction)
// Maintains medical device compliance while following KISS principle

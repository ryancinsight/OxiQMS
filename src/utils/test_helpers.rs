/// DRY and KISS Improvement: Common Test Utilities
/// 
/// This module provides simple test utilities to reduce code duplication
/// in test modules, following both DRY and KISS principles.

#[cfg(test)]
use crate::prelude::*;

#[cfg(test)]
use std::path::PathBuf;

/// Simple test utilities for QMS testing
#[cfg(test)]
pub struct TestHelper;

#[cfg(test)]
impl TestHelper {
    /// Create a temporary directory for testing (DRY: common test setup)
    pub fn create_temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!("qms_test_{}", crate::utils::SimpleUtils::simple_timestamp()))
    }
    
    /// Initialize test audit system (DRY: common audit setup) - KISS: simplified
    pub fn init_audit_for_test() -> QmsResult<()> {
        let temp_dir = Self::create_temp_dir();
        std::fs::create_dir_all(&temp_dir)?;
        // KISS: Just ensure directory exists for testing
        Ok(())
    }
    
    /// Create test risk item (DRY: common test data)
    pub fn create_test_risk() -> crate::models::RiskItem {
        crate::models::RiskItem {
            id: "RISK-TEST-001".to_string(),
            description: "Test risk description".to_string(),
            severity: 3,
            occurrence: 2,
            detectability: 4,
            rpn: 24,
            mitigation: Some("Test mitigation".to_string()),
            created_at: 1234567890,
            updated_at: 1234567890,
        }
    }
    
    /// Create test document (DRY: common test data)
    pub fn create_test_document() -> crate::models::Document {
        crate::models::Document {
            id: "DOC-TEST-001".to_string(),
            title: "Test Document".to_string(),
            content: "Test document content".to_string(),
            version: "1.0.0".to_string(),
            status: crate::models::DocumentStatus::Draft,
            created_at: 1234567890,
            updated_at: 1234567890,
            author: "test_user".to_string(),
            locked: false,
            locked_by: None,
            locked_at: None,
        }
    }
    
    /// Create test requirement (DRY: common test data) - KISS: simplified
    pub fn create_test_requirement() -> crate::models::Requirement {
        crate::models::Requirement {
            id: "REQ-TEST-001".to_string(),
            text: "Test requirement text".to_string(),
            priority: 2, // Medium priority as u8
            status: crate::models::RequirementStatus::Draft,
            created_at: 1234567890,
            updated_at: 1234567890,
        }
    }
    
    /// Create test username (DRY: common test data) - KISS: simplified
    pub fn create_test_username() -> String {
        "test_user".to_string()
    }
    
    /// Create test audit entry (DRY: common test data)
    pub fn create_test_audit_entry() -> crate::models::AuditEntry {
        crate::models::AuditEntry {
            id: "AUDIT-TEST-001".to_string(),
            timestamp: crate::utils::SimpleUtils::simple_timestamp(),
            user_id: "test_user".to_string(),
            session_id: Some("test_session".to_string()),
            action: crate::models::AuditAction::Create,
            entity_type: "Test".to_string(),
            entity_id: "TEST-001".to_string(),
            old_value: None,
            new_value: Some("test_value".to_string()),
            details: Some("Test audit entry".to_string()),
            signature: None,
            checksum: "test_checksum".to_string(),
            previous_hash: None,
            ip_address: Some("127.0.0.1".to_string()),
        }
    }
    
    /// Assert validation result is valid (DRY: common assertion)
    pub fn assert_validation_valid(result: &crate::utils::ValidationResult) {
        assert!(result.is_valid, "Validation should be valid. Errors: {:?}", result.errors);
    }
    
    /// Assert validation result is invalid (DRY: common assertion)
    pub fn assert_validation_invalid(result: &crate::utils::ValidationResult) {
        assert!(!result.is_valid, "Validation should be invalid");
        assert!(!result.errors.is_empty(), "Should have validation errors");
    }
    
    /// Assert QmsResult is Ok (DRY: common assertion)
    pub fn assert_result_ok<T>(result: &QmsResult<T>) {
        assert!(result.is_ok(), "Result should be Ok. Error: {:?}", result.as_ref().err());
    }
    
    /// Assert QmsResult is Err (DRY: common assertion)
    pub fn assert_result_err<T>(result: &QmsResult<T>) {
        assert!(result.is_err(), "Result should be Err");
    }
    
    /// Create test configuration (DRY: common test setup)
    pub fn create_test_config() -> std::collections::HashMap<String, String> {
        let mut config = std::collections::HashMap::new();
        config.insert("log_level".to_string(), "debug".to_string());
        config.insert("audit_retention_days".to_string(), "30".to_string());
        config.insert("max_document_versions".to_string(), "5".to_string());
        config.insert("require_electronic_signature".to_string(), "false".to_string());
        config.insert("backup_enabled".to_string(), "false".to_string());
        config.insert("encryption_enabled".to_string(), "false".to_string());
        config
    }
    
    /// Setup test project structure (DRY: common test setup)
    pub fn setup_test_project() -> QmsResult<PathBuf> {
        let temp_dir = Self::create_temp_dir();
        crate::utils::ConfigHelper::initialize_project_structure(&temp_dir)?;
        Ok(temp_dir)
    }
    
    /// Cleanup test directory (DRY: common test cleanup)
    pub fn cleanup_test_dir(path: &PathBuf) {
        if path.exists() {
            let _ = std::fs::remove_dir_all(path);
        }
    }
    
    /// Create test metrics (DRY: common test data)
    pub fn create_test_metrics() -> crate::utils::SimpleMetrics {
        let mut metrics = crate::utils::SimpleMetrics::new();
        metrics.add_success();
        metrics.add_success();
        metrics.add_failure();
        metrics.add_value(85.5);
        metrics.add_value(92.3);
        metrics.add_value(78.1);
        metrics
    }
    
    /// Assert metrics are reasonable (DRY: common assertion)
    pub fn assert_metrics_reasonable(metrics: &crate::utils::SimpleMetrics) {
        let summary = metrics.summary();
        assert!(summary.total_count > 0, "Should have some data");
        assert!(summary.success_rate >= 0.0 && summary.success_rate <= 100.0, "Success rate should be 0-100%");
        assert!(summary.failure_rate >= 0.0 && summary.failure_rate <= 100.0, "Failure rate should be 0-100%");
    }
    
    /// Create simple test ID (KISS: basic test data)
    pub fn create_simple_test_id(prefix: &str) -> String {
        format!("{}-TEST-001", prefix)
    }
}

/// Simple test validation helper (KISS: basic validation)
#[cfg(test)]
pub struct TestValidation;

#[cfg(test)]
impl TestValidation {
    /// Validate test string is not empty (DRY: common validation)
    pub fn validate_not_empty(value: &str, field_name: &str) -> crate::utils::ValidationResult {
        let mut result = crate::utils::ValidationResult::new();
        if value.is_empty() {
            result.add_error(format!("{} is empty", field_name));
        }
        result
    }

    /// Validate test ID format (DRY: common validation)
    pub fn validate_test_id(id: &str, prefix: &str) -> crate::utils::ValidationResult {
        let mut result = crate::utils::ValidationResult::new();
        if !id.starts_with(prefix) {
            result.add_error(format!("ID should start with {}", prefix));
        }
        result
    }
}

/// Test assertion macros (DRY: reduce test boilerplate)
#[cfg(test)]
#[macro_export]
macro_rules! assert_valid {
    ($validation:expr) => {
        crate::utils::test_helpers::TestHelper::assert_validation_valid(&$validation);
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_invalid {
    ($validation:expr) => {
        crate::utils::test_helpers::TestHelper::assert_validation_invalid(&$validation);
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        crate::utils::test_helpers::TestHelper::assert_result_ok(&$result);
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {
        crate::utils::test_helpers::TestHelper::assert_result_err(&$result);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_helper_creates_valid_test_id() {
        let id = TestHelper::create_simple_test_id("RISK");
        assert_eq!(id, "RISK-TEST-001");

        let validation = TestValidation::validate_test_id(&id, "RISK");
        assert_valid!(validation);
    }
    
    #[test]
    fn test_helper_creates_temp_dir() {
        let temp_dir = TestHelper::create_temp_dir();
        assert!(temp_dir.to_string_lossy().contains("qms_test_"));
    }
    
    #[test]
    fn test_metrics_creation() {
        let metrics = TestHelper::create_test_metrics();
        TestHelper::assert_metrics_reasonable(&metrics);
        
        let summary = metrics.summary();
        assert_eq!(summary.total_count, 3);
        assert!(summary.success_rate > 0.0);
    }
}

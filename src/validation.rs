//! Input validation functions for QMS data
//! Phase 2 infrastructure - comprehensive validation for medical device compliance

#![allow(dead_code)] // Phase 2 infrastructure - validation functions for document control, risk management

// DRY Improvement: Use consolidated validation utilities
use crate::utils::{validate_id_format, validate_range, validate_string_length, CommonValidation};

/// Validation functions for QMS entities
///
/// Validate project name (max 100 characters, not empty)
pub const fn validate_project_name(name: &str) -> bool {
    validate_string_length(name, 100)
}

/// Validate document title (max 200 characters)
pub const fn validate_document_title(title: &str) -> bool {
    validate_string_length(title, 200)
}

/// Validate requirement text (max 1000 characters)
pub const fn validate_requirement_text(text: &str) -> bool {
    validate_string_length(text, 1000)
}

/// Validate test description (max 500 characters)
pub const fn validate_test_description(description: &str) -> bool {
    validate_string_length(description, 500)
}

/// Validate username - DRY: delegates to common utility
pub fn validate_username(username: &str) -> bool {
    CommonValidation::validate_username(username)
}

/// Validate risk severity (1-10)
pub const fn validate_risk_severity(severity: u8) -> bool {
    validate_range(severity, 1, 10)
}

/// Validate risk occurrence (1-10)
pub const fn validate_risk_occurrence(occurrence: u8) -> bool {
    validate_range(occurrence, 1, 10)
}

/// Validate risk detectability (1-10)
pub const fn validate_risk_detectability(detectability: u8) -> bool {
    validate_range(detectability, 1, 10)
}

/// Validate document ID format
pub fn validate_document_id(id: &str) -> bool {
    validate_id_format(id, "DOC-")
}

/// Validate risk ID format
pub fn validate_risk_id(id: &str) -> bool {
    validate_id_format(id, "RISK-")
}

/// Validate requirement ID format
pub fn validate_requirement_id(id: &str) -> bool {
    validate_id_format(id, "REQ-")
}

/// Validate test case ID format
pub fn validate_test_case_id(id: &str) -> bool {
    validate_id_format(id, "TC-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_project_name() {
        assert!(validate_project_name("Valid Project"));
        assert!(!validate_project_name(""));
        assert!(!validate_project_name(&"x".repeat(101)));
    }

    #[test]
    fn test_validate_username() {
        assert!(validate_username("valid_user123"));
        assert!(!validate_username("ab")); // Too short
        assert!(!validate_username(&"x".repeat(51))); // Too long
        assert!(!validate_username("invalid-user")); // Invalid character
    }

    #[test]
    fn test_validate_risk_values() {
        assert!(validate_risk_severity(5));
        assert!(!validate_risk_severity(0));
        assert!(!validate_risk_severity(11));

        assert!(validate_risk_occurrence(3));
        assert!(!validate_risk_occurrence(0));

        assert!(validate_risk_detectability(7));
        assert!(!validate_risk_detectability(11));
    }

    #[test]
    fn test_validate_ids() {
        assert!(validate_document_id("DOC-20250715-001"));
        assert!(!validate_document_id("INVALID-001"));

        assert!(validate_risk_id("RISK-001"));
        assert!(!validate_risk_id("DOC-001"));

        assert!(validate_requirement_id("REQ-001"));
        assert!(validate_test_case_id("TC-001"));
    }
}

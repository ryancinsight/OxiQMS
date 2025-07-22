/// DRY Improvement: Common Validation Utilities
/// 
/// This module consolidates validation logic that was duplicated across
/// multiple modules, following the DRY principle.

use crate::prelude::*;

/// Common validation patterns used throughout the QMS system
pub struct CommonValidation;

impl CommonValidation {
    /// Validate range for numeric values (DRY: used in multiple places)
    pub const fn validate_range(value: u8, min: u8, max: u8) -> bool {
        value >= min && value <= max
    }
    
    /// Validate risk parameter range (1-10) - DRY consolidation
    pub const fn validate_risk_parameter(value: u8) -> bool {
        Self::validate_range(value, 1, 10)
    }
    
    /// Validate severity range (DRY: replaces duplicate validation)
    pub const fn validate_severity(severity: u8) -> bool {
        Self::validate_risk_parameter(severity)
    }
    
    /// Validate occurrence range (DRY: replaces duplicate validation)
    pub const fn validate_occurrence(occurrence: u8) -> bool {
        Self::validate_risk_parameter(occurrence)
    }
    
    /// Validate detectability range (DRY: replaces duplicate validation)
    pub const fn validate_detectability(detectability: u8) -> bool {
        Self::validate_risk_parameter(detectability)
    }
    
    /// Validate string length (DRY: used across multiple modules)
    pub const fn validate_string_length(text: &str, min: usize, max: usize) -> bool {
        let len = text.len();
        len >= min && len <= max
    }
    
    /// Validate non-empty string (DRY: common pattern)
    pub const fn validate_non_empty(text: &str) -> bool {
        !text.is_empty()
    }
    
    /// Validate text content (DRY: common requirement validation)
    pub const fn validate_text_content(text: &str) -> bool {
        Self::validate_non_empty(text) && Self::validate_string_length(text, 1, 1000)
    }
    
    /// Validate ID format with prefix (DRY: used for all entity IDs)
    pub fn validate_id_format(id: &str, prefix: &str) -> bool {
        if !id.starts_with(prefix) {
            return false;
        }
        
        let suffix = &id[prefix.len()..];
        if suffix.len() < 2 {
            return false;
        }
        
        // Check if suffix contains only alphanumeric characters and hyphens
        suffix.chars().all(|c| c.is_alphanumeric() || c == '-')
    }
    
    /// Validate specific entity ID formats (DRY: consolidates all ID validations)
    pub fn validate_document_id(id: &str) -> bool {
        Self::validate_id_format(id, "DOC-")
    }
    
    pub fn validate_risk_id(id: &str) -> bool {
        Self::validate_id_format(id, "RISK-")
    }
    
    pub fn validate_requirement_id(id: &str) -> bool {
        Self::validate_id_format(id, "REQ-")
    }
    
    pub fn validate_test_case_id(id: &str) -> bool {
        Self::validate_id_format(id, "TC-")
    }
    
    pub fn validate_audit_id(id: &str) -> bool {
        Self::validate_id_format(id, "AUDIT-")
    }
    
    /// Validate username format (DRY: common across user management)
    pub fn validate_username(username: &str) -> bool {
        if !Self::validate_string_length(username, 3, 50) {
            return false;
        }
        
        // Username must start with letter and contain only alphanumeric and underscore
        let mut chars = username.chars();
        if let Some(first) = chars.next() {
            first.is_alphabetic() && username.chars().all(|c| c.is_alphanumeric() || c == '_')
        } else {
            false
        }
    }
    
    /// Validate project name (DRY: used in multiple places)
    pub fn validate_project_name(name: &str) -> bool {
        if !Self::validate_string_length(name, 1, 100) {
            return false;
        }
        
        // Project name can contain letters, numbers, spaces, hyphens, and underscores
        name.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_')
    }
    
    /// Validate email format (DRY: basic email validation)
    pub fn validate_email(email: &str) -> bool {
        if !Self::validate_string_length(email, 5, 254) {
            return false;
        }
        
        // Basic email validation: contains @ and has parts before and after
        let parts: Vec<&str> = email.split('@').collect();
        parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() && parts[1].contains('.')
    }
    
    /// Validate file path (DRY: used across file operations)
    pub fn validate_file_path(path: &str) -> bool {
        if path.is_empty() || path.len() > 260 {
            return false;
        }
        
        // Check for invalid characters
        let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
        !path.chars().any(|c| invalid_chars.contains(&c))
    }
    
    /// Validate version string (DRY: used for documents and requirements)
    pub fn validate_version(version: &str) -> bool {
        if version.is_empty() {
            return false;
        }
        
        // Simple semantic version validation: X.Y.Z
        let parts: Vec<&str> = version.split('.').collect();
        parts.len() == 3 && parts.iter().all(|part| {
            !part.is_empty() && part.chars().all(|c| c.is_ascii_digit())
        })
    }
    
    /// Validate date string (DRY: used across multiple modules)
    pub fn validate_date_string(date: &str) -> bool {
        if date.len() != 10 {
            return false;
        }
        
        // Basic YYYY-MM-DD format validation
        let parts: Vec<&str> = date.split('-').collect();
        parts.len() == 3 
            && parts[0].len() == 4 && parts[0].chars().all(|c| c.is_ascii_digit())
            && parts[1].len() == 2 && parts[1].chars().all(|c| c.is_ascii_digit())
            && parts[2].len() == 2 && parts[2].chars().all(|c| c.is_ascii_digit())
    }
}

/// Validation result helper (DRY: common validation result pattern)
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }
    
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
    
    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.is_valid = self.is_valid && other.is_valid;
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_risk_parameter_validation() {
        assert!(CommonValidation::validate_severity(5));
        assert!(!CommonValidation::validate_severity(0));
        assert!(!CommonValidation::validate_severity(11));
        
        assert!(CommonValidation::validate_occurrence(3));
        assert!(CommonValidation::validate_detectability(7));
    }
    
    #[test]
    fn test_id_format_validation() {
        assert!(CommonValidation::validate_risk_id("RISK-001"));
        assert!(CommonValidation::validate_document_id("DOC-2024-001"));
        assert!(!CommonValidation::validate_risk_id("INVALID-001"));
    }
    
    #[test]
    fn test_username_validation() {
        assert!(CommonValidation::validate_username("user123"));
        assert!(CommonValidation::validate_username("test_user"));
        assert!(!CommonValidation::validate_username("123user")); // Can't start with number
        assert!(!CommonValidation::validate_username("us")); // Too short
    }
    
    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new();
        assert!(result.is_valid);
        
        result.add_error("Test error".to_string());
        assert!(!result.is_valid);
        assert!(result.has_errors());
        
        result.add_warning("Test warning".to_string());
        assert!(result.has_warnings());
    }
}
